//! Main application component

use leptos::*;
use leptos_router::*;

/// Main application component
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <div class="min-h-screen bg-gray-900 text-white">
                <Header />
                <main class="container mx-auto px-4 py-8">
                    <Routes>
                        <Route path="/" view=Dashboard />
                        <Route path="/blocks" view=BlockList />
                        <Route path="/block/:id" view=BlockDetail />
                        <Route path="/transactions" view=TransactionList />
                        <Route path="/tx/:id" view=TransactionDetail />
                        <Route path="/account/:address" view=AccountDetail />
                        <Route path="/validators" view=ValidatorList />
                        <Route path="/validator/:id" view=ValidatorDetail />
                        <Route path="/tokens" view=TokenList />
                        <Route path="/token/:mint" view=TokenDetail />
                        <Route path="/search" view=SearchResults />
                        <Route path="/*" view=NotFound />
                    </Routes>
                </main>
                <Footer />
            </div>
        </Router>
    }
}

/// Header component
#[component]
fn Header() -> impl IntoView {
    let (query, set_query) = create_signal(String::new());
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let q = query.get();
        if !q.is_empty() {
            let _ = navigate(&format!("/search?q={}", q), Default::default());
        }
    };

    view! {
        <header class="bg-gray-800 border-b border-gray-700">
            <div class="container mx-auto px-4">
                <div class="flex items-center justify-between h-16">
                    <A href="/" class="text-2xl font-bold text-purple-400">
                        "RandScan"
                    </A>

                    <form on:submit=on_submit class="flex-1 max-w-xl mx-8">
                        <input
                            type="text"
                            placeholder="Search by Block, Transaction, Address..."
                            class="w-full bg-gray-700 text-white rounded-lg px-4 py-2
                                   placeholder-gray-400 focus:outline-none focus:ring-2
                                   focus:ring-purple-500"
                            on:input=move |ev| set_query.set(event_target_value(&ev))
                            prop:value=query
                        />
                    </form>

                    <nav class="flex items-center space-x-6">
                        <A href="/blocks" class="text-gray-300 hover:text-white">"Blocks"</A>
                        <A href="/transactions" class="text-gray-300 hover:text-white">"Transactions"</A>
                        <A href="/validators" class="text-gray-300 hover:text-white">"Validators"</A>
                        <A href="/tokens" class="text-gray-300 hover:text-white">"Tokens"</A>
                    </nav>
                </div>
            </div>
        </header>
    }
}

/// Footer component
#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="bg-gray-800 border-t border-gray-700 py-6">
            <div class="container mx-auto px-4 text-center text-sm text-gray-400">
                "RandScan - RandProtocol Blockchain Explorer"
            </div>
        </footer>
    }
}

/// Dashboard page
#[component]
fn Dashboard() -> impl IntoView {
    let stats = create_resource(
        || (),
        |_| crate::api::get_stats(),
    );

    view! {
        <div class="space-y-8">
            <div class="text-center mb-8">
                <h1 class="text-4xl font-bold mb-2">"RandProtocol Explorer"</h1>
                <p class="text-gray-400">"Privacy-preserving blockchain with dual-token economy"</p>
            </div>

            <Suspense fallback=move || view! { <Loading /> }>
                {move || stats.get().map(|result| match result {
                    Ok(s) => view! {
                        <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                            <StatsCard title="Block Height" value=s.block_height.to_string() />
                            <StatsCard title="Transactions" value=s.total_transactions.to_string() />
                            <StatsCard title="Validators" value=format!("{}/{}", s.active_validators, s.total_validators) />
                            <StatsCard title="Current Epoch" value=s.current_epoch.to_string() />
                        </div>
                    }.into_view(),
                    Err(e) => view! {
                        <div class="text-red-400">{e}</div>
                    }.into_view()
                })}
            </Suspense>
        </div>
    }
}

/// Stats card component
#[component]
fn StatsCard(title: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="bg-gray-800 rounded-lg p-6 border-l-4 border-purple-500">
            <p class="text-gray-400 text-sm">{title}</p>
            <p class="text-2xl font-bold mt-1">{value}</p>
        </div>
    }
}

/// Loading component
#[component]
fn Loading() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center py-12">
            <div class="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-purple-500"></div>
        </div>
    }
}

/// Block list page
#[component]
fn BlockList() -> impl IntoView {
    let (page, set_page) = create_signal(1u32);
    let blocks = create_resource(
        move || page.get(),
        |p| crate::api::get_blocks(p, 20),
    );

    view! {
        <div>
            <h1 class="text-3xl font-bold mb-6">"Blocks"</h1>
            <Suspense fallback=move || view! { <Loading /> }>
                {move || blocks.get().map(|result| match result {
                    Ok(data) => view! {
                        <div class="space-y-3">
                            <For
                                each=move || data.data.clone()
                                key=|b| b.block_id.clone()
                                children=move |b| view! {
                                    <A
                                        href=format!("/block/{}", b.height)
                                        class="block bg-gray-800 rounded-lg p-4 hover:bg-gray-750"
                                    >
                                        <div class="flex justify-between">
                                            <span class="font-bold">{format!("Block #{}", b.height)}</span>
                                            <span class="text-gray-400">{format!("{} txs", b.transaction_count)}</span>
                                        </div>
                                    </A>
                                }
                            />
                        </div>
                        <div class="flex justify-center space-x-4 mt-6">
                            <button
                                class="px-4 py-2 bg-gray-700 rounded disabled:opacity-50"
                                disabled=move || page.get() <= 1
                                on:click=move |_| set_page.update(|p| *p = p.saturating_sub(1))
                            >
                                "Previous"
                            </button>
                            <span class="py-2">{move || format!("Page {}", page.get())}</span>
                            <button
                                class="px-4 py-2 bg-gray-700 rounded"
                                on:click=move |_| set_page.update(|p| *p += 1)
                            >
                                "Next"
                            </button>
                        </div>
                    }.into_view(),
                    Err(e) => view! { <div class="text-red-400">{e}</div> }.into_view()
                })}
            </Suspense>
        </div>
    }
}

/// Block detail page
#[component]
fn BlockDetail() -> impl IntoView {
    let params = use_params_map();
    let block = create_resource(
        move || params.with(|p| p.get("id").cloned().unwrap_or_default()),
        |id| async move { crate::api::get_block(&id).await },
    );

    view! {
        <Suspense fallback=move || view! { <Loading /> }>
            {move || block.get().map(|result| match result {
                Ok(b) => view! {
                    <div class="bg-gray-800 rounded-lg p-6">
                        <h1 class="text-3xl font-bold mb-6">{format!("Block #{}", b.height)}</h1>
                        <div class="grid grid-cols-2 gap-4">
                            <div><span class="text-gray-400">"Block ID: "</span>{b.block_id}</div>
                            <div><span class="text-gray-400">"Epoch: "</span>{b.epoch.to_string()}</div>
                            <div><span class="text-gray-400">"Transactions: "</span>{b.transaction_count.to_string()}</div>
                            <div><span class="text-gray-400">"Finalized: "</span>{if b.finalized { "Yes" } else { "No" }}</div>
                        </div>
                    </div>
                }.into_view(),
                Err(e) => view! { <div class="text-red-400">{e}</div> }.into_view()
            })}
        </Suspense>
    }
}

/// Transaction list page
#[component]
fn TransactionList() -> impl IntoView {
    let (page, _set_page) = create_signal(1u32);
    let transactions = create_resource(
        move || page.get(),
        |p| crate::api::get_transactions(p, 20),
    );

    view! {
        <div>
            <h1 class="text-3xl font-bold mb-6">"Transactions"</h1>
            <Suspense fallback=move || view! { <Loading /> }>
                {move || transactions.get().map(|result| match result {
                    Ok(data) => view! {
                        <div class="space-y-3">
                            {data.data.iter().map(|t| {
                                let tx_id = t.tx_id.clone();
                                let short_id = if tx_id.len() > 16 { format!("{}...", &tx_id[..16]) } else { tx_id.clone() };
                                let tx_type = t.payload_type.to_uppercase();
                                view! {
                                    <A
                                        href=format!("/tx/{}", tx_id)
                                        class="block bg-gray-800 rounded-lg p-4 hover:bg-gray-750"
                                    >
                                        <div class="flex justify-between">
                                            <span class="font-mono">{short_id}</span>
                                            <span class="text-gray-400">{tx_type}</span>
                                        </div>
                                    </A>
                                }
                            }).collect_view()}
                        </div>
                    }.into_view(),
                    Err(e) => view! { <div class="text-red-400">{e}</div> }.into_view()
                })}
            </Suspense>
        </div>
    }
}

/// Transaction detail page
#[component]
fn TransactionDetail() -> impl IntoView {
    let params = use_params_map();
    let transaction = create_resource(
        move || params.with(|p| p.get("id").cloned().unwrap_or_default()),
        |id| async move { crate::api::get_transaction(&id).await },
    );

    view! {
        <Suspense fallback=move || view! { <Loading /> }>
            {move || transaction.get().map(|result| match result {
                Ok(tx) => view! {
                    <div class="bg-gray-800 rounded-lg p-6">
                        <h1 class="text-2xl font-bold mb-6">"Transaction"</h1>
                        <div class="grid grid-cols-2 gap-4">
                            <div><span class="text-gray-400">"TX ID: "</span><span class="font-mono">{tx.tx_id}</span></div>
                            <div><span class="text-gray-400">"Type: "</span>{tx.payload_type.to_uppercase()}</div>
                            <div><span class="text-gray-400">"Status: "</span>{tx.status}</div>
                            <div><span class="text-gray-400">"Fee: "</span>{format!("{:.6} ATLAS", tx.fee as f64 / 1e9)}</div>
                        </div>
                    </div>
                }.into_view(),
                Err(e) => view! { <div class="text-red-400">{e}</div> }.into_view()
            })}
        </Suspense>
    }
}

/// Account detail page
#[component]
fn AccountDetail() -> impl IntoView {
    let params = use_params_map();
    let account = create_resource(
        move || params.with(|p| p.get("address").cloned().unwrap_or_default()),
        |addr| async move { crate::api::get_account(&addr).await },
    );

    view! {
        <Suspense fallback=move || view! { <Loading /> }>
            {move || account.get().map(|result| match result {
                Ok(acc) => view! {
                    <div class="bg-gray-800 rounded-lg p-6">
                        <h1 class="text-2xl font-bold mb-6">"Account"</h1>
                        <p class="font-mono text-sm mb-6">{acc.address}</p>
                        <div class="grid grid-cols-2 gap-4">
                            <div class="bg-gray-700 rounded p-4">
                                <p class="text-gray-400">"ATLAS Balance"</p>
                                <p class="text-2xl font-bold text-purple-400">{format!("{:.4}", acc.atlas_balance_display)}</p>
                            </div>
                            <div class="bg-gray-700 rounded p-4">
                                <p class="text-gray-400">"SHRUG Balance"</p>
                                <p class="text-2xl font-bold text-green-400">{format!("{:.4}", acc.shrug_balance_display)}</p>
                            </div>
                        </div>
                    </div>
                }.into_view(),
                Err(e) => view! { <div class="text-red-400">{e}</div> }.into_view()
            })}
        </Suspense>
    }
}

/// Validator list page
#[component]
fn ValidatorList() -> impl IntoView {
    let (page, _set_page) = create_signal(1u32);
    let validators = create_resource(
        move || page.get(),
        |p| crate::api::get_validators(p, 20),
    );

    view! {
        <div>
            <h1 class="text-3xl font-bold mb-6">"Validators"</h1>
            <Suspense fallback=move || view! { <Loading /> }>
                {move || validators.get().map(|result| match result {
                    Ok(data) => view! {
                        <div class="space-y-3">
                            {data.data.iter().map(|v| {
                                let id = v.validator_id.clone();
                                let stake = format!("{:.2} ATLAS", v.stake_display());
                                let short_id = if id.len() > 16 { format!("{}...", &id[..16]) } else { id.clone() };
                                let status = if v.is_active { "Active" } else { "Inactive" };
                                let status_class = if v.is_active { "text-green-400" } else { "text-gray-400" };
                                view! {
                                    <A
                                        href=format!("/validator/{}", id)
                                        class="block bg-gray-800 rounded-lg p-4 hover:bg-gray-750"
                                    >
                                        <div class="flex justify-between items-center">
                                            <span class="font-mono">{short_id}</span>
                                            <div class="flex items-center space-x-4">
                                                <span>{stake}</span>
                                                <span class=status_class>{status}</span>
                                            </div>
                                        </div>
                                    </A>
                                }
                            }).collect_view()}
                        </div>
                    }.into_view(),
                    Err(e) => view! { <div class="text-red-400">{e}</div> }.into_view()
                })}
            </Suspense>
        </div>
    }
}

/// Validator detail page
#[component]
fn ValidatorDetail() -> impl IntoView {
    let params = use_params_map();
    let validator = create_resource(
        move || params.with(|p| p.get("id").cloned().unwrap_or_default()),
        |id| async move { crate::api::get_validator(&id).await },
    );

    view! {
        <Suspense fallback=move || view! { <Loading /> }>
            {move || validator.get().map(|result| match result {
                Ok(v) => view! {
                    <div class="bg-gray-800 rounded-lg p-6">
                        <h1 class="text-2xl font-bold mb-6">"Validator"</h1>
                        <div class="grid grid-cols-2 gap-4">
                            <div><span class="text-gray-400">"Stake: "</span>{format!("{:.2} ATLAS", v.stake_display)}</div>
                            <div><span class="text-gray-400">"Commission: "</span>{format!("{}%", v.commission_rate)}</div>
                            <div><span class="text-gray-400">"Blocks Produced: "</span>{v.blocks_produced.to_string()}</div>
                            <div><span class="text-gray-400">"Skip Rate: "</span>{format!("{:.2}%", v.skip_rate)}</div>
                        </div>
                    </div>
                }.into_view(),
                Err(e) => view! { <div class="text-red-400">{e}</div> }.into_view()
            })}
        </Suspense>
    }
}

/// Token list page
#[component]
fn TokenList() -> impl IntoView {
    let (page, _set_page) = create_signal(1u32);
    let tokens = create_resource(
        move || page.get(),
        |p| crate::api::get_tokens(p, 20),
    );

    view! {
        <div>
            <h1 class="text-3xl font-bold mb-6">"Tokens"</h1>
            <Suspense fallback=move || view! { <Loading /> }>
                {move || tokens.get().map(|result| match result {
                    Ok(data) => view! {
                        <div class="space-y-3">
                            {data.data.iter().map(|t| {
                                let mint = t.mint_address.clone();
                                let symbol = t.symbol.clone();
                                let name = t.name.clone();
                                let holders = format!("{} holders", t.holder_count);
                                view! {
                                    <A
                                        href=format!("/token/{}", mint)
                                        class="block bg-gray-800 rounded-lg p-4 hover:bg-gray-750"
                                    >
                                        <div class="flex justify-between">
                                            <span class="font-bold">{symbol}" - "{name}</span>
                                            <span class="text-gray-400">{holders}</span>
                                        </div>
                                    </A>
                                }
                            }).collect_view()}
                        </div>
                    }.into_view(),
                    Err(e) => view! { <div class="text-red-400">{e}</div> }.into_view()
                })}
            </Suspense>
        </div>
    }
}

/// Token detail page
#[component]
fn TokenDetail() -> impl IntoView {
    let params = use_params_map();
    let token = create_resource(
        move || params.with(|p| p.get("mint").cloned().unwrap_or_default()),
        |mint| async move { crate::api::get_token(&mint).await },
    );

    view! {
        <Suspense fallback=move || view! { <Loading /> }>
            {move || token.get().map(|result| match result {
                Ok(t) => view! {
                    <div class="bg-gray-800 rounded-lg p-6">
                        <h1 class="text-3xl font-bold mb-6">{t.token}</h1>
                        <div class="grid grid-cols-2 gap-4">
                            <div><span class="text-gray-400">"Total Supply: "</span>{format!("{:.2}", t.total_supply_display)}</div>
                            <div><span class="text-gray-400">"Circulating: "</span>{format!("{:.2}", t.circulating_supply_display)}</div>
                            <div><span class="text-gray-400">"Burned: "</span>{format!("{:.2}", t.burned_display)}</div>
                            <div><span class="text-gray-400">"Holders: "</span>{t.holder_count.to_string()}</div>
                        </div>
                    </div>
                }.into_view(),
                Err(e) => view! { <div class="text-red-400">{e}</div> }.into_view()
            })}
        </Suspense>
    }
}

/// Search results page
#[component]
fn SearchResults() -> impl IntoView {
    let query_params = use_query_map();
    let results = create_resource(
        move || query_params.with(|p| p.get("q").cloned().unwrap_or_default()),
        |q| async move { crate::api::search(&q).await },
    );

    view! {
        <div>
            <h1 class="text-3xl font-bold mb-6">"Search Results"</h1>
            <Suspense fallback=move || view! { <Loading /> }>
                {move || results.get().map(|result| match result {
                    Ok(data) => {
                        if data.is_empty() {
                            view! { <p class="text-gray-400">"No results found"</p> }.into_view()
                        } else {
                            view! {
                                <div class="space-y-3">
                                    <For
                                        each=move || data.clone()
                                        key=|r| format!("{:?}-{}", r.result_type, r.id)
                                        children=move |r| view! {
                                            <A
                                                href=r.url.clone()
                                                class="block bg-gray-800 rounded-lg p-4 hover:bg-gray-750"
                                            >
                                                <span class="font-bold">{r.title}</span>
                                                {r.subtitle.map(|s| view! { <span class="text-gray-400 ml-2">{s}</span> })}
                                            </A>
                                        }
                                    />
                                </div>
                            }.into_view()
                        }
                    },
                    Err(e) => view! { <div class="text-red-400">{e}</div> }.into_view()
                })}
            </Suspense>
        </div>
    }
}

/// 404 page
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="text-center py-20">
            <h1 class="text-6xl font-bold text-purple-400 mb-4">"404"</h1>
            <p class="text-xl text-gray-400 mb-8">"Page not found"</p>
            <A href="/" class="px-6 py-3 bg-purple-600 hover:bg-purple-700 rounded-lg">
                "Return Home"
            </A>
        </div>
    }
}
