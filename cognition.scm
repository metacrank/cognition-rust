(define-module (metacrank cognition)
  #:use-module (guix licenses)
  #:use-module (guix packages)
  #:use-module (guix download)
  #:use-module (guix git)
  #:use-module (guix git-download)
  #:use-module (guix utils)
  #:use-module (guix build-system cargo)
  #:use-module (gnu packages)
  #:use-module (gnu packages crates-io))

(define-public cognition-macros-0.1
  (package
   (name "cognition-macros")
   (version "0.1.1")
   (source
    (origin
     (method git-fetch)
     (uri (git-reference
           (url "https://github.com/metacrank/cognition-rust.git")
           (commit "a336e5d3502d5af887e45c3b671d7f4cfb484c7e")))
     (sha256
      (base32
       "0c2f5ypjqvn0fpcr09av2gv073gdxhq9mla7xcpas8bj6d3c6q6h"))))
   (build-system cargo-build-system)
   (arguments
    `(#:cargo-inputs
      (("proc-macro2" ,rust-proc-macro2-1)
       ("serde" ,rust-quote-1)
       ("syn" ,rust-syn-2))))
   (synopsis "Procedural macros for Cognition.")
   (description
    "Includes the cognition::custom attribute macro, which aids the implementation of custom types in Cognition fllibs.")
   (home-page "https://github.com/metacrank/cognition-rust")
   (license expat)))

(define-public cognition-0.3
  (package
   (name "cognition")
   (version "0.3.1")
   (source
    (origin
     (method git-fetch)
     (uri (git-reference
           (url "https://github.com/metacrank/cognition-rust.git")
           (commit "a336e5d3502d5af887e45c3b671d7f4cfb484c7e")))
     (sha256
      (base32
       "0c2f5ypjqvn0fpcr09av2gv073gdxhq9mla7xcpas8bj6d3c6q6h"))))
   (build-system cargo-build-system)
   (arguments
    `(#:cargo-inputs
      (("libloading" ,rust-libloading-0.8)
       ("serde" ,rust-serde-1)
       ("serde_json" ,rust-serde-json-1)
       ("erased-serde" ,rust-erased-serde-0.4)
       ("cognition-macros" ,cognition-macros-0.1)
       )
      #:phases
      (modify-phases %standard-phases
                     (delete 'package))))
   (synopsis "An unopinionated programming language which offers full publicity of syntax and tokenization.")
   (description
    "Cognition is a fully introspective system designed so that the syntax and hierarchy structure of the
     language is fully public; that is, a file that contains cognition code can alter the way that it is
     being parsed, in real time. Cognition allows for this because the language is completely postfix with
     extremely minimal syntax, and what exists of the syntax can be changed at will. Because the language
     never reads more than it has to, and because the language allows for metaprogramming (talking about
     symbols as if they are data, as well as code), the syntax of the language is made fluid. This allows
     for the advanced manipulation of how the next token is tokenized, and how these tokens are arranged
     into something like the AST without having to explicitly program a rigid syntax.")
   (home-page "https://github.com/metacrank/cognition-rust")
   (license expat)))

cognition-0.3
