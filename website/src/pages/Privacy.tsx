import SEO from '../components/SEO';

export default function Privacy() {
    return (
        <div className="container-custom py-32 max-w-4xl">
            <SEO
                title="Privacy Policy - 100% Local AI | Inboxed"
                description="Our privacy police: We don't want your data. Inboxed runs locally on your device."
                canonical="https://inboxed.email/privacy"
            />
            <h1 className="font-heading text-6xl mb-12">Privacy Policy</h1>

            <div className="space-y-12 font-body text-lg leading-relaxed">
                <section>
                    <h2 className="font-heading text-3xl mb-4 italic">The Short Version</h2>
                    <p>
                        We don't want your data. We don't sell your data. We don't even see your data.
                        Inboxed is designed to run entirely on your device.
                    </p>
                </section>

                <div className="w-full h-px bg-black/20"></div>

                <section>
                    <h2 className="font-heading text-2xl mb-4">1. Local Processing</h2>
                    <p>
                        Inboxed uses Apple MLX to run Large Language Models directly on your Mac.
                        When the AI summarizes an email or categorizes your inbox, that processing happens on your silicon.
                        No email content is ever sent to our servers or third-party AI providers (like OpenAI or Anthropic).
                    </p>
                </section>

                <section>
                    <h2 className="font-heading text-2xl mb-4">2. Data Collection</h2>
                    <p>
                        We collect absolutely zero content data. We do not track:
                    </p>
                    <ul className="list-disc pl-5 mt-4 space-y-2">
                        <li>Email contents or metadata</li>
                        <li>Contacts or addresses</li>
                        <li>AI interactions or prompts</li>
                    </ul>
                    <p className="mt-4">
                        The only data we may collect is anonymous telemetry regarding app crashes or basic usage statistics (like "app opened") to help us improve stability, which you can opt-out of at any time.
                    </p>
                </section>

                <section>
                    <h2 className="font-heading text-2xl mb-4">3. Third Party Services</h2>
                    <p>
                        Inboxed connects directly to your email provider (Gmail, Outlook, IMAP).
                        Authentication happens directly between your device and the provider using OAuth standards.
                        We never see or store your password.
                    </p>
                </section>

                <section>
                    <h2 className="font-heading text-2xl mb-4">4. Updates to this Policy</h2>
                    <p>
                        We may update this policy as the app evolves. Since we don't collect your email address, check this page for updates.
                    </p>
                </section>

                <div className="pt-12 font-mono text-sm text-mutedForeground">
                    Last updated: February 3, 2026
                </div>
            </div>
        </div>
    );
}
