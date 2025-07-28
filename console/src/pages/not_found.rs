use leptos::prelude::*;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div style="text-align: center; padding: 4rem 2rem;">
            <h1 style="font-size: 4rem; font-weight: 700; color: #4f46e5; margin-bottom: 1rem;">
                "404"
            </h1>
            <h2 style="font-size: 1.5rem; font-weight: 600; color: #1f2937; margin-bottom: 1rem;">
                "Page Not Found"
            </h2>
            <p style="color: #6b7280; margin-bottom: 2rem;">
                "The page you're looking for doesn't exist."
            </p>
            <a href="/" class="btn">
                "← Back to Dashboard"
            </a>
        </div>
    }
}
