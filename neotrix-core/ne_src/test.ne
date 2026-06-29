; test.ne — Ne language test suite
(import "ne_src/stdlib.ne")
(import "ne_src/evolve.ne")

; Test VSA operations
(define test-vsa (bind [1 0 1 0] [0 1 0 1]))
(define test-cos (cosine-sim [1 1 0 0] [1 0 1 0])

; Test P0-1: Rotation binding
(define test-role (rotation_seed "noun"))
(define test-rotate-bind (rotation_bind "noun" (random_vector)))
(define test-rotate-unbind (rotation_unbind "noun" test-rotate-bind))

; Test P0-2: Codebook compilation — embed strings to vectors
(define test-embed-foo (embed_string "foo"))
(define test-embed-bar (embed_string "bar"))
(define test-codebook-lookup (codebook_lookup "default" "key")))

; Test mutation primitives
(define test-explore (try-explore "handler_a"))
(define test-exploit (try-exploit "handler_a"))
(define test-repair (try-repair "handler_a"))
(define test-innovate (try-innovate "handler_a" "handler_b"))
(define test-harden (try-harden "handler_a"))
(define test-prune (try-prune "handler_a"))
(define test-evolve-cycle (evolve-cycle "handler_a"))
