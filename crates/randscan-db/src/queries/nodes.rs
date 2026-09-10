use crate::{NodeGeoRow, Result};
use sqlx::PgPool;

pub async fn get_node_geo(pool: &PgPool, ips: &[String]) -> Result<Vec<NodeGeoRow>> {
    Ok(sqlx::query_as::<_, NodeGeoRow>(
        "SELECT ip, lat, lon, city, region, country, country_code, org, ok, updated_at FROM node_geo WHERE ip = ANY($1)",
    )
    .bind(ips)
    .fetch_all(pool)
    .await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn upsert_node_geo(
    pool: &PgPool,
    ip: &str,
    ok: bool,
    lat: Option<f64>,
    lon: Option<f64>,
    city: Option<&str>,
    region: Option<&str>,
    country: Option<&str>,
    country_code: Option<&str>,
    org: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO node_geo (ip, lat, lon, city, region, country, country_code, org, ok, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
         ON CONFLICT (ip) DO UPDATE SET lat = EXCLUDED.lat, lon = EXCLUDED.lon, city = EXCLUDED.city,
            region = EXCLUDED.region, country = EXCLUDED.country, country_code = EXCLUDED.country_code,
            org = EXCLUDED.org, ok = EXCLUDED.ok, updated_at = NOW()",
    )
    .bind(ip)
    .bind(lat)
    .bind(lon)
    .bind(city)
    .bind(region)
    .bind(country)
    .bind(country_code)
    .bind(org)
    .bind(ok)
    .execute(pool)
    .await?;
    Ok(())
}
