import { Check, X, Shield, Zap } from 'lucide-react';
import { Link } from 'react-router-dom';
import SEO from '../components/SEO';

export default function ComparisonSuperhuman() {
    return (
        <div className="container-custom py-32">
            <SEO
                title="Superhuman Alternative (Free & Local) | Inboxed"
                description="Switch from Superhuman to Inboxed. Save $360/year and keep your data private with local AI processing."
                canonical="https://inboxed.email/compare/superhuman"
            />
            <div className="max-w-4xl mx-auto">
                <h1 className="font-serif text-5xl md:text-7xl mb-8">Inboxed vs. Superhuman</h1>
                <p className="font-body text-xl md:text-2xl mb-16 max-w-2xl leading-relaxed">
                    Superhuman is fast. But it sends your data to the cloud and costs $360/year.
                    Inboxed runs locally on your Mac and costs $1 for life.
                </p>

                <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-20">
                    {/* Feature Column */}
                    <div className="hidden md:block col-span-1 pt-24 font-mono text-sm uppercase tracking-widest text-mutedForeground space-y-10">
                        <div>AI Processing</div>
                        <div>Privacy</div>
                        <div>Cost</div>
                        <div>Platform</div>
                        <div>Data Access</div>
                    </div>

                    {/* Superhuman Card */}
                    <div className="p-8 border-2 border-muted bg-muted/20 opacity-70">
                        <h3 className="font-serif text-2xl mb-2">Superhuman</h3>
                        <div className="h-1 w-10 bg-black/20 mb-10"></div>

                        <div className="space-y-10 font-sans text-lg">
                            <div className="flex items-center gap-2">
                                <X size={20} className="text-black/50" />
                                <span>Cloud API (OpenAI)</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <X size={20} className="text-black/50" />
                                <span>Data leaves device</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <span className="font-bold">$30/month</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <span>Web / Cloud Wrapper</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <X size={20} className="text-black/50" />
                                <span>3rd Party Access</span>
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
                                <span>Local LLM (Apple MLX)</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <Shield size={20} className="text-black" />
                                <span>100% Private</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <Zap size={20} className="text-black" />
                                <span>$1 Lifetime (Pro)</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <span>Native macOS App</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <Check size={20} className="text-black" />
                                <span>Zero Data Sharing</span>
                            </div>
                        </div>
                    </div>
                </div>

                <div className="border-t border-black pt-16">
                    <h2 className="font-serif text-4xl mb-6">Why switch?</h2>
                    <p className="font-body text-lg leading-relaxed mb-4">
                        The era of sending your personal emails to a third-party server for "AI features" is over.
                        Apple Silicon is powerful enough to run intelligent models right on your laptop.
                    </p>
                    <p className="font-body text-lg leading-relaxed mb-8">
                        Save $359 a year. Get better privacy. Own your software.
                    </p>

                    <Link to="/" className="btn-primary inline-block">
                        Get Inboxed for $1
                    </Link>
                </div>
            </div>
        </div>
    );
}
