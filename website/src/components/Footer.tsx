import { Link } from 'react-router-dom';

export default function Footer() {
    return (
        <footer className="bg-black text-white pt-24 pb-12 border-t-8 border-black">
            <div className="container-custom">
                <div className="grid grid-cols-1 md:grid-cols-4 gap-12 mb-20">
                    <div className="col-span-1 md:col-span-2">
                        <h2 className="font-serif text-5xl mb-6">inboxed.email</h2>
                        <p className="font-body text-lg text-white/70 max-w-md">
                            Reclaiming the inbox for the modern era. Private, local, and intelligent email for macOS.
                        </p>
                    </div>

                    <div>
                        <h4 className="font-mono text-xs uppercase tracking-widest mb-6 text-white/50">Product</h4>
                        <ul className="space-y-4 font-body">
                            <li><a href="#" className="hover:underline underline-offset-4">Download</a></li>
                            <li><a href="#" className="hover:underline underline-offset-4">Changelog</a></li>
                            <li><a href="#" className="hover:underline underline-offset-4">Roadmap</a></li>
                            <li><Link to="/blog" className="hover:underline underline-offset-4">Blog</Link></li>
                        </ul>
                    </div>

                    <div>
                        <h4 className="font-mono text-xs uppercase tracking-widest mb-6 text-white/50">Legal</h4>
                        <ul className="space-y-4 font-body">
                            <li><Link to="/privacy" className="hover:underline underline-offset-4">Privacy Policy</Link></li>
                            <li><Link to="/terms" className="hover:underline underline-offset-4">Terms of Service</Link></li>
                        </ul>
                    </div>

                    <div>
                        <h4 className="font-mono text-xs uppercase tracking-widest mb-6 text-white/50">Compare</h4>
                        <ul className="space-y-4 font-body">
                            <li><Link to="/compare/superhuman" className="hover:underline underline-offset-4">vs Superhuman</Link></li>
                            <li><Link to="/compare/zero" className="hover:underline underline-offset-4">vs 0.email</Link></li>
                        </ul>
                    </div>
                </div>

                <div className="border-t border-white/20 pt-8 flex flex-col md:flex-row justify-between items-center gap-4">
                    <p className="font-mono text-xs text-white/50">
                        © 2024 Inboxed Inc. All rights reserved.
                    </p>
                    <p className="font-mono text-xs text-white/50">
                        Designed in California. Built with Apple MLX.
                    </p>
                </div>
            </div>
        </footer>
    );
}
