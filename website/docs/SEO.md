# Inboxed — Day-1 SEO README

This README documents the **SEO strategy and operating principles** for launching and growing **Inboxed**, a local-AI email client.

It is intended to be:
- An internal reference for founders and contributors
- A checklist to ensure SEO is done correctly **from day 1**
- A guardrail to avoid common early-stage SEO mistakes

---

## 1. SEO Foundations (Before Launch)

### Domain & Canonical Setup

- Enforce **HTTPS only**
- Choose a single canonical domain:
  - `https://inboxed.email`
- Configure redirects:
  - `http → https`
  - `www → non-www` (or the reverse, but never both)

### Search Engine Setup

- Verify the site in **Google Search Console**
- Submit:
  - XML sitemap (`/sitemap.xml`)
- Enable:
  - Indexing
  - Core Web Vitals reporting

---

## 2. Site Structure

### URL Principles

Keep URLs **flat, readable, and stable**.

**Recommended**

/
/features/local-ai
/pricing
/compare/superhuman
/blog/local-ai-email

**Avoid**

/product/email/ai/local/features/page1

### Navigation Rules

- All important pages reachable within **2 clicks**
- No orphan pages
- Footer must include:
  - About
  - Privacy
  - Comparisons

---

## 3. Required Pages at Launch

Minimum SEO-viable page set:

1. **Homepage**
   - Primary keyword: `local AI email client`
   - Secondary keyword: `Superhuman alternative`

2. **Features**
   - Local AI processing
   - Offline support
   - Privacy & security

3. **Pricing**

4. **Comparison pages**
   - `/superhuman-alternative`
   - (Later) `/shortwave-alternative`, `/gmail-ai-alternative`

5. **Privacy Policy**
   - Mandatory for trust and E-E-A-T

6. **About Page**
   - Founder context
   - Product philosophy

---

## 4. On-Page SEO Rules

### Title Tags

- 50–60 characters
- Brand name at the end

Example:

Local AI Email Client | Inboxed

### Meta Descriptions

- 140–160 characters
- Focus on outcomes, not features

Example:

Inboxed is a local AI email client that works offline, respects privacy, and replaces cloud AI email tools.

### Headings

- Exactly **one H1 per page**
- H1 contains the primary keyword
- Use H2/H3 for structure only

---

## 5. Content Strategy

### High-Intent Keywords (Early Wins)

These queries can rank even for new domains:

- `superhuman alternative`
- `offline email client`
- `local ai email`
- `private ai email client`

Each should map to a **dedicated page**, not a generic blog post.

---

### Blog Content Guidelines

Early blog posts should be **technical, opinionated, and specific**:

- Why local AI beats cloud AI for email
- How offline semantic search works
- The privacy risks of cloud email AI
- Why Superhuman-style tools fail privacy audits

Avoid generic or filler content.

---

## 6. Technical SEO Essentials

### Performance Targets

- LCP < 2.5s
- CLS < 0.1
- INP < 200ms

### Implementation Notes

- Use server-side rendering for marketing pages
- Compress images (WebP / AVIF)
- Avoid heavy JS frameworks for landing pages

---

## 7. Structured Data (Schema)

Add early:

- `Organization`
- `SoftwareApplication`
- `FAQ` (especially on comparison pages)

Benefits:
- Improved CTR
- Rich results
- Stronger trust signals

---

## 8. Internal Linking Rules

- Every page links to **at least 2 other pages**
- Use descriptive anchor text

Good:
> Superhuman alternative with local AI

Bad:
> Click here

---

## 9. Trust & E-E-A-T Signals

Required from day 1:

- Real founder name
- Clear contact method
- Transparent privacy explanation
- No fake testimonials
- No stock team imagery

Trust compounds SEO over time.

---

## 10. Anti-Patterns (Do Not Do These)

- Keyword stuffing
- Mass AI-generated content
- Buying backlinks
- Launching with a single page
- Changing domain or URL structure post-launch

---

## 11. 30-Day SEO Roadmap

### Week 1
- Launch site
- Submit sitemap
- Publish Superhuman comparison page

### Week 2
- Publish 2 technical blog posts
- Add FAQ schema

### Week 3
- Improve internal linking
- Refine titles and descriptions

### Week 4
- Publish one long-form alternative article
- Begin light ranking tracking

---

## 12. Competitive Landscape

Inboxed competes in the **AI-assisted email** space. These products influence search intent and should have comparison or alternative pages over time.

### Full AI Email Clients (Cloud-Based)

- Superhuman
- Shortwave
- Gmail (AI features)
- Outlook with Copilot

### AI Email Wrappers / Assistants

- AI reply generators
- Writing assistants
- CRM-embedded email AI

These are **not full clients**, but they shape user expectations and queries.

### Inboxed Positioning

- Local-first AI (on-device)
- Offline-capable
- Privacy-preserving by design
- No cloud training on user emails

This positioning must be consistent across:
- Homepage
- Comparison pages
- Blog content
- Metadata

---

## Bottom Line

Inboxed SEO success depends on:
- Clear positioning
- Clean structure
- Comparison-driven intent
- Trust and transparency

This README is the **SEO source of truth** for the project.
