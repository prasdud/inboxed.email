import { Routes, Route, useLocation } from 'react-router-dom';
import { useEffect } from 'react';
import Navbar from './components/Navbar';
import Footer from './components/Footer';
import Home from './pages/Home';
import Privacy from './pages/Privacy';
import Terms from './pages/Terms';
import ComparisonSuperhuman from './pages/ComparisonSuperhuman';
import ComparisonZero from './pages/ComparisonZero';
import BlogIndex from './pages/BlogIndex';
import PostLocalAI from './pages/blog/PostLocalAI';

// Scroll to top on route change
function ScrollToTop() {
  const { pathname } = useLocation();

  useEffect(() => {
    window.scrollTo(0, 0);
  }, [pathname]);

  return null;
}

function App() {
  return (
    <div className="bg-background text-foreground min-h-screen selection:bg-black selection:text-white flex flex-col">
      <ScrollToTop />
      <Navbar />
      <main className="flex-grow">
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/privacy" element={<Privacy />} />
          <Route path="/terms" element={<Terms />} />
          <Route path="/compare/superhuman" element={<ComparisonSuperhuman />} />
          <Route path="/compare/zero" element={<ComparisonZero />} />
          <Route path="/blog" element={<BlogIndex />} />
          <Route path="/blog/local-ai-email" element={<PostLocalAI />} />
        </Routes>
      </main>
      <Footer />
    </div>
  )
}

export default App
