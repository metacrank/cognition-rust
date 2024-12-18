(define-module (metacrank cognition)
  #:use-module (guix licenses)
  #:use-module (guix packages)
  #:use-module (guix download)
  #:use-module (guix git)
  #:use-module (guix git-download)
  #:use-module (guix utils)
  #:use-module (gnu packages)
  #:use-module (guix build-system copy))

(define-public cogsh-0.1
  (package
   (name "cogsh")
   (version "0.1.0")
   (source
    (origin
     (method git-fetch)
     (uri (git-reference
           (url "https://github.com/metacrank/cognition-rust.git")
           (commit "f665258ee23895c38da42656b9fd8c734843f2f7")))
     (sha256
      (base32
       "1bk751x11mwcm70pl2q2s89b3y7m9aw0px2xkq50njhsl3j53ccg"))))
   (build-system copy-build-system)
   (synopsis "A shell based on the Cognition programming language.")
   (description
    "Cogsh is a shell based on the Cognition programming language. It requires the env and process fllibs. It does not yet support Bash syntax.")
   (home-page "https://github.com/metacrank/cognition-rust")
   (license expat)))

cogsh-0.1
