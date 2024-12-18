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
           (commit "03cf05bd2ba08d2c44b299cf5736a2707d71cd94")))
     (sha256
      (base32
       "0dy7kam835l2znqpapm0gc5by4bx513r3wmznvmca0rp8679889j"))))
   (build-system copy-build-system)
   (arguments
    `(#:install-plan `(("coglib/utils/cogsh.cog" "bin/cogsh"))))
   (synopsis "A shell based on the Cognition programming language.")
   (description
    "Cogsh is a shell based on the Cognition programming language. It requires the env and process fllibs. It does not yet support Bash syntax.")
   (home-page "https://github.com/metacrank/cognition-rust")
   (license expat)))

cogsh-0.1
