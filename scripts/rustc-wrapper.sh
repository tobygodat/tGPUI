#!/bin/sh
# Cargo runs this in place of rustc. It links with mold and caches through
# sccache when they are installed, and otherwise runs rustc unchanged.
# mold goes in as a constant flag, not a per-checkout linker path, because
# sccache hashes every argument and worktrees would never share a hit. Cross
# builds and explicit linkers are left alone: rust-lld rejects the flag.
case " $* " in
  *" --target "* | *linker=*) ;;
  *) command -v mold >/dev/null 2>&1 && set -- "$@" -C link-arg=-fuse-ld=mold ;;
esac
command -v sccache >/dev/null 2>&1 && exec sccache "$@"
exec "$@"
