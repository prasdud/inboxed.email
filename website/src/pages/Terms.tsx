import SEO from '../components/SEO';

export default function Terms() {
    return (
        <div className="container-custom py-32 max-w-4xl">
            <SEO
                title="Terms of Service | Inboxed"
                description="Terms of Service for Inboxed, the local AI email client."
                canonical="https://inboxed.email/terms"
            />
            <h1 className="font-heading text-6xl mb-12">Terms of Service</h1>

            <div className="space-y-12 font-body text-lg leading-relaxed">
                <section>
                    <h2 className="font-heading text-2xl mb-4">1. Acceptance of Terms</h2>
                    <p>
                        By downloading or using Inboxed ("the Application"), you agree to be bound by these Terms of Service.
                        If you disagree with any part of the terms, you may not use the Application.
                    </p>
                </section>

                <section>
                    <h2 className="font-heading text-2xl mb-4">2. License</h2>
                    <p>
                        Inboxed grants you a revocable, non-exclusive, non-transferable, limited license to download, install,
                        and use the Application solely for your personal, non-commercial purposes strictly in accordance with the terms of this Agreement.
                    </p>
                </section>

                <section>
                    <h2 className="font-heading text-2xl mb-4">3. Local AI Usage</h2>
                    <p>
                        The Application utilizes local machine learning models ("Apple MLX") to process data.
                        You acknowledge that while these models run locally, AI predictions may not always be 100% accurate.
                        You are responsible for verifying critical actions taken by the AI (such as archiving or deleting emails).
                    </p>
                </section>

                <section>
                    <h2 className="font-heading text-2xl mb-4">4. Disclaimer</h2>
                    <p>
                        The Application is provided "AS IS" without warranties of any kind.
                        We do not warrant that the Application will function uninterrupted or be error-free.
                    </p>
                </section>

                <section>
                    <h2 className="font-heading text-2xl mb-4">5. limitation of Liability</h2>
                    <p>
                        In no event shall Inboxed be liable for any indirect, incidental, special, consequential or punitive damages, including without limitation, loss of profits, data, use, goodwill, or other intangible losses, resulting from your access to or use of or inability to access or use the Application.
                    </p>
                </section>

                <div className="pt-12 font-mono text-sm text-mutedForeground">
                    Last updated: February 3, 2026
                </div>
            </div>
        </div>
    );
}
