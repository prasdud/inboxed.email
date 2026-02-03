import { Cpu, Zap, EyeOff, Lock } from 'lucide-react';


const features = [
    {
        icon: <Cpu strokeWidth={1} size={32} />,
        title: "Local Private LLM",
        description: "Powered by Apple MLX. Your AI assistant runs entirely on your silicon. No API costs, no latency, no data leaks."
    },
    {
        icon: <EyeOff strokeWidth={1} size={32} />,
        title: "No Tracking",
        description: "We don't track opens, clicks, or your data. Your email is your business. We rely on your device's power, not our servers."
    },
    {
        icon: <Zap strokeWidth={1} size={32} />,
        title: "Zero Inbox",
        description: "AI automatically categorizes newsletters, receipts, and updates. Focus only on what matters. Clear your inbox in seconds."
    },
    {
        icon: <Lock strokeWidth={1} size={32} />,
        title: "Direct Fetch",
        description: "Emails are fetched directly from Google/IMAP to your device. No middleman servers reading your correspondence."
    }
];

export default function Features() {
    return (
        <section id="features" className="py-32 bg-background border-t-8 border-black">
            <div className="container-custom">
                <div className="mb-20">
                    <h2 className="font-serif text-6xl md:text-7xl mb-6">Features.</h2>
                    <p className="font-mono text-sm uppercase tracking-widest text-mutedForeground">
                        Designed for the privacy-conscious professional.
                    </p>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-0 border border-black">
                    {features.map((feature, index) => (
                        <div key={index}
                            className={`
                   group p-12 border-black
                   ${index % 2 === 0 ? 'md:border-r' : ''}
                   ${index < features.length - (features.length % 2 === 0 ? 2 : 1) ? 'border-b' : ''}
                   hover:bg-black hover:text-white transition-colors duration-200
                 `}>
                            <div className="mb-8 p-4 border border-black inline-block rounded-none group-hover:border-white group-hover:bg-white group-hover:text-black transition-colors duration-200">
                                {feature.icon}
                            </div>
                            <h3 className="font-serif text-3xl mb-4 italic">{feature.title}</h3>
                            <p className="font-body text-lg leading-relaxed opacity-80 group-hover:opacity-100">
                                {feature.description}
                            </p>
                        </div>
                    ))}
                </div>

                <div className="mt-32 border-t-2 border-black/10 pt-16">
                    <h3 className="font-serif text-3xl md:text-5xl mb-12 text-center">Engineered for Silicon.</h3>

                    <div className="grid grid-cols-1 md:grid-cols-3 gap-12 text-center">
                        <div>
                            <div className="font-mono font-bold text-xl mb-4">Rust + Tauri v2</div>
                            <p className="font-body text-black/70 leading-relaxed">
                                Built on a secure, memory-safe backend. The app binary is tiny (~10MB) and uses a fraction of the RAM of Electron apps.
                            </p>
                        </div>
                        <div>
                            <div className="font-mono font-bold text-xl mb-4">Apple Metal</div>
                            <p className="font-body text-black/70 leading-relaxed">
                                We use <span className="font-bold">llama.cpp</span> optimized for Metal to run 7B+ parameter models directly on your GPU.
                            </p>
                        </div>
                        <div>
                            <div className="font-mono font-bold text-xl mb-4">Zero Data Exit</div>
                            <p className="font-body text-black/70 leading-relaxed">
                                Your credentials stay in the macOS Keychain. Your data stays in a local SQLite db. No servers. No analytics.
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    );
}
