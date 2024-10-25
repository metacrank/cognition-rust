(define-module (metacrank cognition)
  #:use-module (guix licenses)
  #:use-module (guix packages)
  #:use-module (guix download)
  #:use-module (guix git)
  #:use-module (guix git-download)
  #:use-module (guix utils)
  #:use-module (guix build-system cargo)
  #:use-module (gnu packages))

(define-public cognition
  (package
   (name "cognition")
   (version "0.1.0")
   (source
    (origin
     (method git-fetch)
     (uri (git-reference
           (url "https://github.com/metacrank/cognition-rust.git")
           (commit "e675d26861c71e52283966168ef2082b3c2228c2")))
     (sha256
      (base32
       "0wgz34m315fxn949a2ciwkqn3dsc5sd85kvssigbd1yvll4qkyd6"))
     ;;(hash (content-hash "05zxkyz9bv3j9h0xyid1rhvh3klhsmrpkf3bcs6frvlgyr2gwilj"))
     ))
   (build-system cargo-build-system)
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

cognition
