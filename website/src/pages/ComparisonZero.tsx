import { Check, X, Zap } from 'lucide-react';
import { Link } from 'react-router-dom';
import SEO from '../components/SEO';

export default function ComparisonZero() {
    return (
        <div className="container-custom py-32">
            <SEO
                title="0.email Alternative (Native Mac App) | Inboxed"
                description="The best native alternative to 0.email. Zero setup, native Apple Silicon performance, and full local AI privacy."
                canonical="https://inboxed.email/compare/zero"
            />
            <div className="max-w-4xl mx-auto">
                <h1 className="font-serif text-5xl md:text-7xl mb-8">Inboxed vs. 0.email</h1>
                <p className="font-body text-xl md:text-2xl mb-16 max-w-2xl leading-relaxed">
                    0.email is a great open-source project. But it runs in your browser or as a web wrapper.
                    Inboxed is a high-performance PRO app built natively for Apple Silicon.
                </p>

                <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-20">
                    {/* Feature Column */}
                    <div className="hidden md:block col-span-1 pt-24 font-mono text-sm uppercase tracking-widest text-mutedForeground space-y-10">
                        <div>Platform</div>
                        <div>Architecture</div>
                        <div>Ease of Use</div>
                        <div>Cost</div>
                    </div>

                    {/* 0.email Card */}
                    <div className="p-8 border-2 border-muted bg-muted/20 opacity-70">
                        <h3 className="font-serif text-2xl mb-2">0.email</h3>
                        <div className="h-1 w-10 bg-black/20 mb-10"></div>

                        <div className="space-y-10 font-sans text-lg">
                            <div className="flex items-center gap-2">
                                <X size={20} className="text-black/50" />
                                <span>Web / Browser-based</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <X size={20} className="text-black/50" />
                                <span>Javascript / Web Stack</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <X size={20} className="text-black/50" />
                                <span>Self-host / Open Source</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <span>Free (Open Source)</span>
                            </div>
                        </div>
                    </div>

                    {/* Inboxed Card */}
                    <div className="p-8 border-4 border-black bg-white relative">
                        <div className="absolute -top-4 left-1/2 -translate-x-1/2 bg-black text-white px-4 py-1 text-xs uppercase tracking-widest">
                            Winner
                        </div>
                        <h3 className="font-serif text-2xl mb-2">Inboxed</h3>
                        <div className="h-1 w-10 bg-black mb-10"></div>

                        <div className="space-y-10 font-sans text-lg font-medium">
                            <div className="flex items-center gap-2">
                                <Check size={20} className="text-black" />
                                <span>Native macOS (Swift/Rust)</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <Check size={20} className="text-black" />
                                <span>Apple MLX Native</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <Check size={20} className="text-black" />
                                <span>One-Click Install</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <Zap size={20} className="text-black" />
                                <span>$1 Lifetime (Pro)</span>
                            </div>
                        </div>
                    </div>
                </div>

                <div className="border-t border-black pt-16">
                    <h2 className="font-serif text-4xl mb-6">Native Power. Zero Friction.</h2>
                    <p className="font-body text-lg leading-relaxed mb-8">
                        Web apps can feel sluggish. Inboxed is optimized for your Mac's hardware.
                        Get the polish of a native app with the intelligence of a local LLM.
                    </p>

                    <Link to="/" className="btn-primary inline-block">
                        Get Inboxed for $1
                    </Link>
                </div>
            </div>
        </div>
    );
}
