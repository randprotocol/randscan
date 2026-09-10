//! Tracks the explorer node's peers and geolocates their public IPs (nodes map).

use crate::rpc::RpcClient;
use anyhow::Result;
use randscan_core::{is_private_ip, parse_multiaddr, NodeInfo};
use randscan_db::{self as db, DbPool};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Re-lookup an IP after this long.
const GEO_TTL: Duration = Duration::from_secs(30 * 24 * 3600);

pub struct NodeTracker {
    rpc: RpcClient,
    pool: DbPool,
    http: reqwest::Client,
    interval: Duration,
    public_ip: RwLock<Option<String>>,
    nodes: Arc<RwLock<Vec<NodeInfo>>>,
}

#[derive(Debug, Deserialize)]
struct IpWhoIs {
    #[serde(default)]
    success: bool,
    #[serde(default)]
    ip: String,
    #[serde(default)]
    city: Option<String>,
    #[serde(default)]
    region: Option<String>,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    country_code: Option<String>,
    #[serde(default)]
    latitude: Option<f64>,
    #[serde(default)]
    longitude: Option<f64>,
    #[serde(default)]
    connection: Option<IpWhoIsConnection>,
}

#[derive(Debug, Deserialize)]
struct IpWhoIsConnection {
    #[serde(default)]
    org: Option<String>,
    #[serde(default)]
    isp: Option<String>,
}

impl NodeTracker {
    pub fn new(rpc: RpcClient, pool: DbPool, interval: Duration) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("randscan/0.2 (+https://randscan.org)")
            .build()
            .expect("reqwest client");
        Self {
            rpc,
            pool,
            http,
            interval,
            public_ip: RwLock::new(
                std::env::var("NODE_PUBLIC_IP")
                    .ok()
                    .filter(|s| !s.is_empty()),
            ),
            nodes: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn nodes(&self) -> Vec<NodeInfo> {
        self.nodes.read().await.clone()
    }

    pub async fn run(&self) {
        loop {
            if let Err(e) = self.refresh().await {
                warn!("node tracker refresh failed: {:#}", e);
            }
            tokio::time::sleep(self.interval).await;
        }
    }

    async fn refresh(&self) -> Result<()> {
        let status = self.rpc.status().await?;
        let peers = self.rpc.peers().await?;

        let self_ip = self.discover_public_ip().await;
        let mut nodes: Vec<NodeInfo> = Vec::with_capacity(peers.len() + 1);
        nodes.push(NodeInfo {
            peer_id: status.peer_id.clone(),
            ip: self_ip,
            port: Some(30303),
            connected_secs: None,
            is_self: true,
            role: if status.is_validator {
                "validator"
            } else {
                "observer"
            }
            .into(),
            geo: None,
        });
        for p in peers {
            // Prefer a public address; peers behind a VPC may list a private one first.
            let parsed: Vec<(Option<String>, Option<u16>)> =
                p.addrs.iter().map(|a| parse_multiaddr(a)).collect();
            let (ip, port) = parsed
                .iter()
                .find(|(ip, _)| ip.as_deref().map(|i| !is_private_ip(i)).unwrap_or(false))
                .or_else(|| parsed.iter().find(|(ip, _)| ip.is_some()))
                .cloned()
                .unwrap_or((None, None));
            nodes.push(NodeInfo {
                peer_id: p.peer_id,
                ip,
                port,
                connected_secs: Some(p.connected_secs as i64),
                is_self: false,
                role: "peer".into(),
                geo: None,
            });
        }

        let ips: Vec<String> = nodes
            .iter()
            .filter_map(|n| n.ip.clone())
            .filter(|ip| !is_private_ip(ip))
            .collect();
        let mut cache: HashMap<String, db::NodeGeoRow> = db::get_node_geo(self.pool.inner(), &ips)
            .await?
            .into_iter()
            .map(|r| (r.ip.clone(), r))
            .collect();

        for ip in &ips {
            let stale = cache
                .get(ip)
                .map(|r| {
                    chrono_age(r.updated_at) > GEO_TTL
                        || (!r.ok && chrono_age(r.updated_at) > Duration::from_secs(3600))
                })
                .unwrap_or(true);
            if stale {
                match self.lookup(ip).await {
                    Ok(row) => {
                        cache.insert(ip.clone(), row);
                    }
                    Err(e) => warn!("geo lookup {} failed: {:#}", ip, e),
                }
            }
        }

        for n in nodes.iter_mut() {
            if let Some(ip) = &n.ip {
                n.geo = cache.get(ip).and_then(|r| r.geo());
            }
        }
        // Peers reached over a private network (e.g. a cloud VPC) sit in the same place as this node.
        let self_geo = nodes.iter().find(|n| n.is_self).and_then(|n| n.geo.clone());
        if let Some(sg) = self_geo {
            for n in nodes.iter_mut() {
                if n.geo.is_none() && n.ip.as_deref().map(is_private_ip).unwrap_or(false) {
                    n.geo = Some(randscan_core::GeoInfo {
                        org: Some("private network of this node".into()),
                        ..sg.clone()
                    });
                }
            }
        }

        debug!(
            "node tracker: {} nodes, {} geolocated",
            nodes.len(),
            nodes.iter().filter(|n| n.geo.is_some()).count()
        );
        *self.nodes.write().await = nodes;
        Ok(())
    }

    async fn discover_public_ip(&self) -> Option<String> {
        if let Some(ip) = self.public_ip.read().await.clone() {
            return Some(ip);
        }
        match self.http.get("https://ipwho.is/").send().await {
            Ok(resp) => match resp.json::<IpWhoIs>().await {
                Ok(me) if me.success && !me.ip.is_empty() => {
                    info!("public ip of this node: {}", me.ip);
                    *self.public_ip.write().await = Some(me.ip.clone());
                    Some(me.ip)
                }
                _ => None,
            },
            Err(e) => {
                warn!("public ip discovery failed: {}", e);
                None
            }
        }
    }

    async fn lookup(&self, ip: &str) -> Result<db::NodeGeoRow> {
        let r: IpWhoIs = self
            .http
            .get(format!("https://ipwho.is/{}", ip))
            .send()
            .await?
            .json()
            .await?;
        let org = r
            .connection
            .as_ref()
            .and_then(|c| c.org.clone().or_else(|| c.isp.clone()));
        let ok = r.success && r.latitude.is_some() && r.longitude.is_some();
        db::upsert_node_geo(
            self.pool.inner(),
            ip,
            ok,
            r.latitude,
            r.longitude,
            r.city.as_deref(),
            r.region.as_deref(),
            r.country.as_deref(),
            r.country_code.as_deref(),
            org.as_deref(),
        )
        .await?;
        Ok(db::NodeGeoRow {
            ip: ip.to_string(),
            lat: r.latitude,
            lon: r.longitude,
            city: r.city,
            region: r.region,
            country: r.country,
            country_code: r.country_code,
            org,
            ok,
            updated_at: chrono_now(),
        })
    }
}

fn chrono_now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

fn chrono_age(t: chrono::DateTime<chrono::Utc>) -> Duration {
    (chrono::Utc::now() - t).to_std().unwrap_or_default()
}
