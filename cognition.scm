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
           (commit "60719559e00b7d88e5c8e1a8424abeb4d1d44381")))
     (sha256
      (base32
       "17fymcpyyjlwapjpx7h74fy8glb086ijkcpjl7kw8an894070lgj"))))
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
   (version "0.3.3")
   (source
    (origin
     (method git-fetch)
     (uri (git-reference
           (url "https://github.com/metacrank/cognition-rust.git")
           (commit "4c073843593b8ef9fc8c95e69e55a8a837b9e125")))
     (sha256
      (base32
       "0hqh12v63hw3b9i0slq85bwfbinclgkc6h8iamhh1lybi33xhfwv"))))
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
