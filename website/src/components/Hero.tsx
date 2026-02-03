import { ArrowDown } from 'lucide-react';

export default function Hero() {
    return (
        <section className="relative min-h-screen flex flex-col justify-center pt-20 pb-20 overflow-hidden bg-background">
            {/* Background Pattern - Horizontal Lines */}
            <div className="absolute inset-0 opacity-[0.03] pointer-events-none"
                style={{ backgroundImage: 'repeating-linear-gradient(0deg, transparent, transparent 1px, #000 1px, #000 2px)', backgroundSize: '100% 4px' }}>
            </div>

            <div className="container-custom relative z-10">
                <div className="flex flex-col gap-6">
                    <h1 className="font-serif leading-[0.9] text-6xl md:text-8xl lg:text-9xl uppercase tracking-tight">
                        Your Inbox. <br />
                        <span className="italic">Private.</span> <br />
                        Intelligent.
                    </h1>

                    <div className="w-full h-1 bg-black my-6"></div> {/* Thick Rule */}

                    <div className="grid grid-cols-1 md:grid-cols-12 gap-12 items-start">
                        <div className="md:col-span-12 lg:col-span-5">
                            <p className="font-body text-lg md:text-xl leading-relaxed text-black/90">
                                The first desktop email client powered by a <span className="font-bold border-b border-black">Local Private LLM</span>.
                                Organize your emails, achieve Zero Inbox, and automate workflows—all without your data ever leaving your device.
                            </p>
                        </div>

                        <div className="md:col-span-12 lg:col-span-7 flex flex-col items-start gap-8">
                            <div className="flex flex-wrap gap-4">
                                <button className="btn-primary">
                                    Download for Mac
                                </button>
                                <button className="btn-outline">
                                    Read Manifesto
                                </button>
                            </div>
                            <div className="font-mono text-xs text-mutedForeground uppercase tracking-widest mt-4">
                                Powered by Apple MLX • macOS Sequoia Ready
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            {/* Scroll Indicator */}
            <div className="absolute bottom-10 left-1/2 -translate-x-1/2 animate-bounce">
                <ArrowDown size={24} strokeWidth={1} />
            </div>
        </section>
    );
}
