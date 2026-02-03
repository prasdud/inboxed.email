import { Download } from 'lucide-react';
import { Link } from 'react-router-dom';

export default function Navbar() {
    return (
        <nav className="fixed top-0 left-0 right-0 z-50 bg-background/90 backdrop-blur-sm border-b border-black">
            <div className="container-custom flex items-center justify-between h-20">
                <div className="flex items-center gap-2">
                    {/* Minimal Logo */}
                    <Link to="/" className="w-8 h-8 bg-black flex items-center justify-center">
                        <span className="text-white font-serif italic text-lg font-bold">i</span>
                    </Link>
                    <Link to="/" className="font-serif text-2xl font-bold tracking-tight">inboxed.email</Link>
                </div>

                <div className="hidden md:flex items-center gap-8">
                    <Link to="/#features" className="text-sm uppercase tracking-widest hover:underline hover:decoration-1 underline-offset-4">Features</Link>
                    <Link to="/privacy" className="text-sm uppercase tracking-widest hover:underline hover:decoration-1 underline-offset-4">Privacy</Link>
                </div>

                <div>
                    <button className="bg-black text-white px-6 py-2 text-xs uppercase tracking-widest font-medium hover:bg-white hover:text-black hover:border hover:border-black transition-colors duration-100 flex items-center gap-2">
                        <span>Download</span>
                        <Download size={14} />
                    </button>
                </div>
            </div>
        </nav>
    );
}
