use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::types::SystemStats;

#[component]
pub fn HomePage() -> impl IntoView {
    let (stats, set_stats) = signal(None::<SystemStats>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    // Load stats on mount
    spawn_local(async move {
        set_loading.set(true);
        match api::get_system_stats().await {
            Ok(system_stats) => {
                set_stats.set(Some(system_stats));
                set_error.set(None);
            }
            Err(err) => {
                set_error.set(Some(err));
            }
        }
        set_loading.set(false);
    });

    view! {
        <div class="dashboard-grid">
            {move || {
                if loading.get() {
                    view! {
                        <div class="loading">
                            <div class="spinner"></div>
                            "Loading stats..."
                        </div>
                    }.into_any()
                } else if let Some(err) = error.get() {
                    view! {
                        <div class="error">
                            "Failed to load stats: " {err}
                        </div>
                    }.into_any()
                } else if let Some(system_stats) = stats.get() {
                    view! {
                        <StatsCards stats=system_stats/>
                    }.into_any()
                } else {
                    view! {
                        <div>"No data available"</div>
                    }.into_any()
                }
            }}
        </div>
        
        <div class="dashboard-grid">
            <div class="dashboard-card">
                <div class="card-header">
                    <h3 class="card-title">"Quick Actions"</h3>
                </div>
                <div style="display: flex; gap: 1rem; flex-wrap: wrap;">
                    <a href="/buckets" class="btn">
                        "📦 Manage Buckets"
                    </a>
                    <a href="/settings" class="btn btn-secondary">
                        "⚙️ Settings"
                    </a>
                </div>
            </div>
            
            <div class="dashboard-card">
                <div class="card-header">
                    <h3 class="card-title">"Recent Activity"</h3>
                </div>
                <p class="card-description">"Recent uploads, downloads, and bucket operations will appear here."</p>
            </div>
        </div>
    }
}

#[component]
fn StatsCards(stats: SystemStats) -> impl IntoView {
    let format_size = |bytes: u64| {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    };

    view! {
        <div class="dashboard-card">
            <div class="card-header">
                <h3 class="card-title">"Total Buckets"</h3>
            </div>
            <div class="card-value">{stats.total_buckets}</div>
            <p class="card-description">"Storage containers"</p>
        </div>
        
        <div class="dashboard-card">
            <div class="card-header">
                <h3 class="card-title">"Total Objects"</h3>
            </div>
            <div class="card-value">{stats.total_objects}</div>
            <p class="card-description">"Stored files"</p>
        </div>
        
        <div class="dashboard-card">
            <div class="card-header">
                <h3 class="card-title">"Storage Used"</h3>
            </div>
            <div class="card-value">{format_size(stats.total_size_bytes)}</div>
            <p class="card-description">{format!("{:.1}% of capacity", stats.storage_usage_percent)}</p>
        </div>
    }
}
