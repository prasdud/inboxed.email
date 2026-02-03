import { Link } from 'react-router-dom';
import SEO from '../components/SEO';

const posts = [
    {
        slug: '/blog/local-ai-email',
        title: 'Why Local Private LLMs are the Future of Email',
        date: 'February 3, 2026',
        excerpt: 'Stop sending your personal data to the cloud. Learn why on-device intelligence is faster, safer, and better for professional communication.'
    }
];

export default function BlogIndex() {
    return (
        <div className="container-custom py-32 max-w-4xl">
            <SEO
                title="Inboxed Blog - Local Private LLMs & Privacy"
                description="Insights on local AI, privacy-first software, and the future of email."
                canonical="https://inboxed.email/blog"
            />
            <h1 className="font-serif text-6xl mb-16">Blog.</h1>

            <div className="grid gap-12">
                {posts.map(post => (
                    <div key={post.slug} className="group">
                        <Link to={post.slug} className="block">
                            <div className="flex flex-col md:flex-row gap-6 md:items-baseline mb-4">
                                <h2 className="font-serif text-3xl font-bold group-hover:underline decoration-2 underline-offset-4">{post.title}</h2>
                                <span className="font-mono text-sm text-mutedForeground whitespace-nowrap">{post.date}</span>
                            </div>
                            <p className="font-body text-xl text-black/80 leading-relaxed max-w-2xl">
                                {post.excerpt}
                            </p>
                            <div className="mt-4 font-mono text-sm uppercase tracking-widest text-black group-hover:translate-x-2 transition-transform inline-block">
                                Read Article &rarr;
                            </div>
                        </Link>
                    </div>
                ))}
            </div>
        </div>
    );
}
