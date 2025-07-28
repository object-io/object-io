use leptos::prelude::*;
use leptos_meta::*;

mod components;
mod pages;
mod api;
mod types;

use pages::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="ObjectIO Console"/>
        <Meta charset="utf-8"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1"/>
        <Meta name="description" content="ObjectIO - S3-compatible storage management console"/>
        
        <div class="app">
            <Header/>
            <main class="main-content">
                <HomePage/>
            </main>
        </div>
    }
}

#[component]
fn Header() -> impl IntoView {
    view! {
        <header class="header">
            <div class="header-content">
                <div class="logo">
                    "ObjectIO"
                </div>
                <nav class="nav">
                    <span class="nav-link">"Dashboard"</span>
                    <span class="nav-link">"Buckets"</span>
                    <span class="nav-link">"Settings"</span>
                </nav>
            </div>
        </header>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
