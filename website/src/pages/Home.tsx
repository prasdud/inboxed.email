import Hero from '../components/Hero';
import Features from '../components/Features';
import { Link } from 'react-router-dom';
import SEO from '../components/SEO';

export default function Home() {
    return (
        <>
            <SEO
                title="Local Private LLM Email Client | Inboxed"
                description="Inboxed is a local private LLM email client that works offline, respects privacy, and replaces cloud AI email tools."
                schema={{
                    "@context": "https://schema.org",
                    "@type": "SoftwareApplication",
                    "name": "Inboxed",
                    "applicationCategory": "Productivity",
                    "operatingSystem": "macOS",
                    "offers": {
                        "@type": "Offer",
                        "price": "1.00",
                        "priceCurrency": "USD"
                    }
                }}
            />
            <Hero />
            <Features />
            {/* Pricing Section */}
            <section className="py-32 border-t-8 border-black bg-muted/30">
                <div className="container-custom">
                    <h2 className="font-serif text-5xl md:text-7xl mb-16 text-center">Fair Pricing.</h2>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-8 max-w-4xl mx-auto mb-20">
                        {/* Free Tier */}
                        <div className="p-8 border-2 border-black bg-white flex flex-col">
                            <h3 className="font-serif text-3xl mb-2">Standard</h3>
                            <div className="text-4xl font-mono font-bold mb-6">Free</div>
                            <ul className="space-y-4 font-body text-lg mb-8 flex-grow">
                                <li className="flex gap-2"><span className="text-black font-bold">✓</span> Basic Local AI</li>
                                <li className="flex gap-2"><span className="text-black font-bold">✓</span> Unlimited Accounts</li>
                                <li className="flex gap-2"><span className="text-black font-bold">✓</span> Community Support</li>
                            </ul>
                            <button className="btn-outline w-full">Download</button>
                        </div>

                        {/* Pro Tier */}
                        <div className="p-8 border-4 border-black bg-black text-white flex flex-col relative transform md:-translate-y-4 shadow-[10px_10px_0px_0px_rgba(0,0,0,0.2)]">
                            <div className="absolute top-0 right-0 bg-white text-black text-xs font-bold px-3 py-1 uppercase tracking-widest">
                                Best Value
                            </div>
                            <h3 className="font-serif text-3xl mb-2">Pro Lifetime</h3>
                            <div className="text-4xl font-mono font-bold mb-6">$1 <span className="text-sm font-normal opacity-70">/ life</span></div>
                            <ul className="space-y-4 font-body text-lg mb-8 flex-grow opacity-90">
                                <li className="flex gap-2"><span>✓</span> Advanced MLX Models</li>
                                <li className="flex gap-2"><span>✓</span> Priority Support</li>
                                <li className="flex gap-2"><span>✓</span> Early Access Features</li>
                                <li className="flex gap-2"><span>✓</span> Support Development</li>
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
                            <Link to="/compare/superhuman" className="hover:underline">vs Superhuman</Link>
                            <Link to="/compare/zero" className="hover:underline">vs 0.email</Link>
                        </div>
                    </div>
                </div>
            </section>

            {/* Statement Section */}
            <section className="py-40 border-t-8 border-black">
                <div className="container-custom text-center">
                    <h2 className="font-heading text-5xl md:text-8xl mb-12 italic">
                        "The email client for the AI era."
                    </h2>
                    <p className="font-body text-xl md:text-2xl max-w-3xl mx-auto leading-relaxed">
                        Stop sending your personal data to the cloud to get smart features.
                        Inboxed brings the power of Large Language Models directly to your device.
                    </p>
                </div>
            </section>
        </>
    );
}
