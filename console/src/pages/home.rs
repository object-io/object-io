use leptos::prelude::*;
use crate::api;
use crate::types::SystemStats;

#[component]
pub fn HomePage() -> impl IntoView {
    let stats = leptos::prelude::Resource::new(|| (), |_| api::get_system_stats());

    view! {
        <div class="dashboard-grid">
            <Suspense fallback=move || view! { <div class="loading"><div class="spinner"></div>"Loading stats..."</div> }>
                {move || {
                    stats.get().map(|result| {
                        match result {
                            Ok(stats) => view! {
                                <StatsCards stats=stats/>
                            }.into_view(),
                            Err(err) => view! {
                                <div class="error">
                                    "Failed to load stats: " {err}
                                </div>
                            }.into_view(),
                        }
                    })
                }}
            </Suspense>
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
