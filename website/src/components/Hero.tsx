import { ArrowDown } from 'lucide-react';

export default function Hero() {
    return (
        <section className="relative min-h-screen flex flex-col justify-between pt-24 pb-16 overflow-hidden bg-background">
            {/* Background Pattern - Horizontal Lines */}
            <div
                className="absolute inset-0 opacity-[0.03] pointer-events-none"
                style={{
                    backgroundImage:
                        'repeating-linear-gradient(0deg, transparent, transparent 1px, #000 1px, #000 2px)',
                    backgroundSize: '100% 4px',
                }}
            />

            {/* Hero Content */}
            <div className="container-custom relative z-10 flex-1 flex flex-col justify-center">
                <div className="flex flex-col gap-8">
                    {/* Overline Label */}
                    <div
                        className="font-mono text-xs text-mutedForeground uppercase tracking-[0.3em] animate-[fadeInUp_0.6s_ease-out_both]"
                    >
                        Private AI Email Client
                    </div>

                    {/* Headline */}
                    <h1
                        className="font-serif leading-[0.88] text-6xl md:text-8xl lg:text-9xl uppercase tracking-tight animate-[fadeInUp_0.6s_ease-out_0.1s_both]"
                    >
                        Your Inbox.
                        <br />
                        <span className="italic font-normal">Private AI.</span>
                        <br />
                        On-Device.
                    </h1>

                    {/* Thick Rule */}
                    <div
                        className="w-full h-[3px] bg-black my-2 animate-[fadeInUp_0.6s_ease-out_0.2s_both]"
                    />

                    {/* Description + CTAs */}
                    <div
                        className="grid grid-cols-1 lg:grid-cols-12 gap-10 lg:gap-16 items-start animate-[fadeInUp_0.6s_ease-out_0.3s_both]"
                    >
                        <div className="lg:col-span-5">
                            <p className="font-body text-lg md:text-xl leading-relaxed text-foreground/85">
                                Inboxed is the first <span className="font-bold">private email client</span> powered by a{' '}
                                <span className="font-bold border-b-2 border-black">
                                    Local Private LLM
                                </span>
                                . Achieve zero inbox with on-device AI summarization—all without your data ever leaving your
                                device.
                            </p>
                        </div>

                        <div className="lg:col-span-7 flex flex-col items-start gap-6">
                            <div className="flex flex-wrap gap-4">
                                <button className="btn-primary">
                                    Download for Mac
                                </button>
                                {/* <button className="btn-outline">
                                    Read Manifesto
                                </button> */}
                            </div>
                            <div className="font-mono text-xs text-mutedForeground uppercase tracking-widest">
                                Powered by Apple MLX &bull;  llama.cpp
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            {/* App Screenshot Mockup */}
            <div
                className="container-custom relative z-10 mt-16 md:mt-20 animate-[fadeInUp_0.8s_ease-out_0.5s_both]"
            >
                <div className="relative mx-auto max-w-5xl">
                    {/* Shadow layer */}
                    <div className="absolute inset-0 translate-y-4 bg-black/10 blur-2xl" />

                    {/* Window frame */}
                    <div className="relative border-2 border-black bg-background">
                        {/* Title bar */}
                        <div className="flex items-center gap-2 px-4 py-3 border-b-2 border-black bg-muted">
                            {/* Traffic lights */}
                            <span className="w-3 h-3 border-2 border-black bg-foreground" />
                            <span className="w-3 h-3 border-2 border-black bg-foreground" />
                            <span className="w-3 h-3 border-2 border-black bg-foreground" />
                            <span className="ml-4 font-mono text-[11px] text-mutedForeground uppercase tracking-widest select-none">
                                MailBox -- Smart Inbox
                            </span>
                        </div>

                        {/* Window content */}
                        <div className="relative flex items-center justify-center bg-muted/50 aspect-[16/9] overflow-hidden">
                            {/* Grid pattern inside window */}
                            <div
                                className="absolute inset-0 opacity-[0.04] pointer-events-none"
                                style={{
                                    backgroundImage:
                                        'linear-gradient(#000 1px, transparent 1px), linear-gradient(90deg, #000 1px, transparent 1px)',
                                    backgroundSize: '40px 40px',
                                }}
                            />

                            {/* Placeholder content */}
                            <div className="relative flex flex-col items-center gap-6 p-8">
                                {/* Fake UI skeleton lines */}
                                <div className="flex flex-col gap-3 w-full max-w-md">
                                    <div className="flex items-center gap-3">
                                        <div className="w-8 h-8 bg-black/10 shrink-0" />
                                        <div className="h-3 bg-black/10 flex-1" />
                                        <div className="h-3 bg-black/10 w-16" />
                                    </div>
                                    <div className="flex items-center gap-3">
                                        <div className="w-8 h-8 bg-black/15 shrink-0" />
                                        <div className="h-3 bg-black/15 flex-1" />
                                        <div className="h-3 bg-black/15 w-12" />
                                    </div>
                                    <div className="flex items-center gap-3">
                                        <div className="w-8 h-8 bg-black/10 shrink-0" />
                                        <div className="h-3 bg-black/10 flex-1" />
                                        <div className="h-3 bg-black/10 w-20" />
                                    </div>
                                </div>

                                <div className="font-mono text-sm md:text-base text-mutedForeground uppercase tracking-[0.25em] select-none">
                                    App Screenshot
                                </div>

                                <div className="flex flex-col gap-3 w-full max-w-md">
                                    <div className="flex items-center gap-3">
                                        <div className="w-8 h-8 bg-black/10 shrink-0" />
                                        <div className="h-3 bg-black/10 flex-1" />
                                        <div className="h-3 bg-black/10 w-14" />
                                    </div>
                                    <div className="flex items-center gap-3">
                                        <div className="w-8 h-8 bg-black/10 shrink-0" />
                                        <div className="h-3 bg-black/10 flex-1" />
                                        <div className="h-3 bg-black/10 w-10" />
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            {/* Scroll Indicator */}
            <div className="absolute bottom-8 left-1/2 -translate-x-1/2 animate-bounce">
                <ArrowDown size={24} strokeWidth={1} />
            </div>

            {/* Keyframe animation */}
            <style>{`
                @keyframes fadeInUp {
                    from {
                        opacity: 0;
                        transform: translateY(24px);
                    }
                    to {
                        opacity: 1;
                        transform: translateY(0);
                    }
                }
            `}</style>
        </section>
    );
}
