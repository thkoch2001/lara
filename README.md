Lara (**L**ocal **Ar**chive, **A**) wants to be(come) a

- decentralized
- web search engine
- web archive
- and feed reader.

Project status: incomplete, experimental

A user should be able to run an instance of Lara on their own (or rented)
hardware and connect it with other trusted remote instances.

A search query combines results from own and remote instances.

# Assumptions

Instances only crawl and index sites handpicked and trusted by their
administrators. This eases the implementation:

- Crawled sites can be assumed to not be spam or intentionally misbehaving.
- Ranking of sites is not a priority anymore since all of them are considered
  valuable. Thus a big webgraph and pagerank is not needed.
- Most Instances run on one single machine but instances can also scale to
  small clusters.

# Motivation

It is widely acknowledged, that the quality of Google search has declined:

- The term "enshittification"[^3] has been created for this phenomenon.
- lightning talk at Chaos Communication Congress 2023[^4]

[^3]: 2023-07-28, Cory Doctorow: [Microincentives and Enshittification](https://pluralistic.net/2023/07/28/microincentives-and-enshittification)
[^4]: 2023-12-28, Martin Hamilton: [Honey I federated the search engine - finding stuff online post-big tech](https://media.ccc.de/v/37c3-lightningtalks-58060-honey-i-federated-the-search-engine-finding-stuff-online-post-big-tech)

The paper [GOGGLES: Democracy dies in darkness, and so does the Web][Goggles]
by the Brave Search Team discusses the issue of biases in search engines.

[Goggles]: https://brave.com/static-assets/files/goggles.pdf

The [Trusted News Initiative][5] (Wikipedia!) is an alliance of Big Tech and
others to combat what they consider disinformation.

[5]: https://en.wikipedia.org/wiki/Trusted_News_Initiative

See also my blog post [Rebuild search with trust][6] with further motivation
and links to previous decentralized search projects, especially
[YaCy](https://yacy.net).

[6]: https://blog.koch.ro/posts/2024-01-20-rebuild-search-with-trust.html

The project should also serve as an archive:

> “Who controls the past controls the future: who controls the present
> controls the past” --- 1984, George Orwell

# LaraBot

LaraBot is the crawler of this project. It tries to be polite. Please be
patient. Open an issue in this project or contact larabot@koch.ro for any
feedback. Feedback is welcome!

# Misc

- Github? Git repos can be moved quickly. Github provides visibility.
- "Lara"? Provisional name. My [second dog](https://photos.app.goo.gl/7u7NUC4iX6Dp5o986).
