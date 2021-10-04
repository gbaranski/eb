<h1 align="center">
  <a href="https://github.com/gbaranski/eb">eb - Editor Backend</a>
</h1>
<h4 align="center">Experimental backend for code editors following a client server architecture.</h4>

- [Architecture](#architecture)
- [Project status](#project-status)
- [Problem](#problem)
- [Q&A](#qa)
  - [Why not Vim over SSH/Mosh?](#why-not-vim-over-sshmosh)
  - [What's different from Kakoune server?](#whats-different-from-kakoune-server)
        - [Why not `xi-editor`?](#why-not-xi-editor)


# Architecture

![Architecture](docs/architecture.png)

# Project status

I'm trying to develop a working proof-of-concept.

# Problem

No separation of client/server in current editors.

Separating editors to client and a server has few advanteges: 

1. Server can be started on a beefy machine while running frontend on a small and slow laptop, without slowing down your development experience or consuming battery.
2. Easy collaborative programming.
3. Easier to create a new frontend, no need for handling LSP Server yourself. Some could say "Why would you want to implement a new fronted? We already have X."
  - Imagine you want a native MacOS editor made with Cocoa, or a editor made in GTK, there are not that many of those, and they might not support advanced features such as LSP.
  - I'm a Vim user myself, and I don't imagine coding in a editor which doesn't support Vim keybindings, some editors such as VSCode have plugins for that, however from my experience they make the editor horribly slow. 
4. Long-running sessions. You can run the server in background, while having editor closed, that'd greatly improve it's startup time. 


# Q&A

## Why not Vim over SSH/Mosh?

I've been doing this for a long time, this works, but it has few cons:
- Quite high input latency.
- Mosh doesn't support true color, unless we use `master` branch which isn't available on Termux.
- No native feeling. Not a big issue on Laptop/PC, but on mobile it is hard to use.

## What's different from Kakoune server?

[Kakoune](https://github.com/mawww/kakoune) makes server used only by the kakoune frontend. `eb` wants to be something that can be used by many editors such as VSCode. Similarly how Microsoft came up with LSP which is now used by most of the editors right now. 

##### Why not `xi-editor`?

`xi-editor` is dead, but I'd like to take some inspiration from it's design.
