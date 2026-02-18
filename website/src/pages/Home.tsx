import Hero from '../components/Hero';
import Features from '../components/Features';
import { Link } from 'react-router-dom';
import SEO from '../components/SEO';

export default function Home() {
    return (
        <>
            <SEO
                title="Private AI Email Client for Mac | Local LLM | Inboxed"
                description="Inboxed is the premier private email client for macOS. Run on-device AI for email summarization and triage without the cloud. Secure, offline, and fast."
                schemas={[
                    {
                        "@context": "https://schema.org",
                        "@type": "SoftwareApplication",
                        "name": "Inboxed",
                        "description": "A high-performance private email client for macOS. Inboxed uses local LLMs to provide intelligent features like email summarization and smart categorization entirely on-device using Apple MLX and llama.cpp. Supports 7B+ parameter models offline.",
                        "applicationCategory": "BusinessApplication",
                        "operatingSystem": "macOS",
                        "softwareVersion": "1.0",
                        "fileSize": "10MB",
                        "downloadUrl": "https://inboxed.email",
                        "featureList": "Free Superhuman alternative, Private AI email summarization, Local LLM processing, Offline email client capabilities, Apple Metal GPU acceleration, Secure IMAP fetch, Zero-data-exit privacy",
                        "offers": [
                            {
                                "@type": "Offer",
                                "name": "Standard",
                                "price": "0",
                                "priceCurrency": "USD",
                                "description": "Free private email client with local AI models, unlimited accounts, and community support"
                            },
                            {
                                "@type": "Offer",
                                "name": "Pro Lifetime",
                                "price": "1.00",
                                "priceCurrency": "USD",
                                "description": "One-time payment for priority support and early access to new private AI features"
                            }
                        ]
                    },
                    {
                        "@context": "https://schema.org",
                        "@type": "Organization",
                        "name": "Inboxed",
                        "url": "https://inboxed.email",
                        "description": "Maker of Inboxed, the local private LLM email client for macOS.",
                        "brand": {
                            "@type": "Brand",
                            "name": "Inboxed"
                        }
                    }
                ]}
            />
            <Hero />
            <Features />
            {/* Pricing Section */}
            <section className="py-32 border-t-8 border-black bg-muted/30">
                <div className="container-custom">
                    <h2 className="font-serif text-5xl md:text-7xl mb-4 text-center">Fair Pricing.</h2>
                    <p className="font-mono text-xs uppercase tracking-widest text-mutedForeground text-center mb-16">
                        No subscriptions. No hidden costs.
                    </p>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-8 max-w-4xl mx-auto mb-20">
                        {/* Free Tier */}
                        <div className="p-8 border-2 border-black bg-white flex flex-col">
                            <h3 className="font-serif text-3xl mb-2">Standard</h3>
                            <div className="text-4xl font-mono font-bold mb-4">Free</div>
                            <div className="w-full h-px bg-black/20 mb-6" />
                            <ul className="space-y-4 font-body text-lg mb-8 flex-grow">
                                <li className="flex gap-3 items-baseline"><span className="text-black font-bold">&mdash;</span> All AI Models</li>
                                <li className="flex gap-3 items-baseline"><span className="text-black font-bold">&mdash;</span> Unlimited Accounts</li>
                                <li className="flex gap-3 items-baseline"><span className="text-black font-bold">&mdash;</span> Community Support</li>
                            </ul>
                            <button className="btn-outline w-full">Download</button>
                        </div>

                        {/* Pro Tier */}
                        <div className="p-8 border-4 border-black bg-black text-white flex flex-col relative transform md:-translate-y-4 transition-shadow duration-200 hover:shadow-[8px_8px_0px_0px_#000]">
<h3 className="font-serif text-3xl mb-2">Pro Lifetime</h3>
                            <div className="text-4xl font-mono font-bold mb-4">$1 <span className="text-sm font-normal opacity-70">/ life</span></div>
                            <div className="w-full h-px bg-white/20 mb-6" />
                            <ul className="space-y-4 font-body text-lg mb-8 flex-grow opacity-90">
                                <li className="flex gap-3 items-baseline"><span>&mdash;</span> Everything in Standard</li>
                                <li className="flex gap-3 items-baseline relative group">
                                    <span>&mdash;</span> Local fine-tuning (LoRA) using MLX
                                    <span className="inline-flex items-center justify-center w-4 h-4 rounded-full border border-white/50 text-white/50 text-[10px] font-mono cursor-default ml-1 shrink-0 self-center">?</span>
                                    <div className="absolute bottom-full left-0 mb-3 md:bottom-auto md:top-1/2 md:left-full md:-translate-y-1/2 md:mb-0 md:ml-3 w-56 bg-white text-black p-4 shadow-[4px_4px_0px_0px_rgba(255,255,255,0.3)] opacity-0 pointer-events-none group-hover:opacity-100 group-hover:pointer-events-auto md:opacity-100 md:pointer-events-auto transition-opacity duration-200 z-10">
                                        <p className="font-body text-sm leading-snug font-semibold mb-1">Your own private AI model trained on your emails.</p>
                                        <p className="font-body text-xs leading-relaxed text-black/60">Runs and trains locally on your device.</p>
                                    </div>
                                </li>
                                <li className="flex gap-3 items-baseline"><span>&mdash;</span> Support Development</li>
                            </ul>
                            <button className="bg-white text-black px-8 py-4 text-sm font-medium tracking-widest uppercase hover:bg-gray-200 transition-colors w-full">
                                Get Pro
                            </button>
                        </div>
                    </div>

                    <div className="text-center">
                        <p className="font-mono text-xs uppercase tracking-widest text-mutedForeground mb-8">
                            See how we compare
                        </p>
                        <div className="flex flex-col md:flex-row justify-center gap-4 md:gap-12 font-body text-lg underline-offset-4">
                            <Link to="/compare/superhuman" className="hover:underline text-black font-bold">Free Superhuman Alternative</Link>
                            <Link to="/compare/canary-mail" className="hover:underline">vs Canary Mail</Link>
                        </div>
                    </div>
                </div>
            </section>

            {/* Statement Section */}
            <section className="py-40 border-t-8 border-black">
                <div className="container-custom text-center">
                    <div className="font-serif text-[8rem] md:text-[12rem] leading-none select-none text-black/10 -mb-16 md:-mb-24">
                        &ldquo;
                    </div>
                    <h2 className="font-heading text-5xl md:text-8xl mb-12 italic">
                        "The email client for the AI era."
                    </h2>
                    <p className="font-body text-xl md:text-2xl max-w-3xl mx-auto leading-relaxed mb-16">
                        Stop sending your personal data to the cloud to get smart features.
                        Inboxed brings the power of Large Language Models directly to your device.
                    </p>
                    <div className="w-24 h-[4px] bg-black mx-auto mb-16" />
                    <button className="btn-primary">
                        Download for Mac
                    </button>
                </div>
            </section>
        </>
    );
}
