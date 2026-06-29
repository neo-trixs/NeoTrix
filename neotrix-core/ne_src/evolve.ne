; evolve.ne — Self-evolution rules for NeoTrix consciousness (v2)
; Uses CI-injected env vars: cycle, handler-count, anomaly-score, anomaly-trained
; Each rule proposes a mutation based on handler state

; Export the mutation functions for CI runtime
(export)

; — Helper: predicate primitives from CI env —
(define (anomaly> threshold)
  (> anomaly-score threshold)
)

(define (even-cycle?)
  (= 0 (- cycle (* 2 (/ cycle 2))))
)

(define (many-handlers? n)
  (> handler-count n)
)

; — Rule 1: explore — activate a handler (high handlers / high anomaly) —
(define (try-explore name)
  (explore name)
)

; — Rule 2: exploit — optimize a handler (stable conditions) —
(define (try-exploit name)
  (exploit name)
)

; — Rule 3: repair — restore a handler (high anomaly needs repair) —
(define (try-repair name)
  (repair name)
)

; — Rule 4: innovate — combine two handlers (low handlers need new combos) —
(define (try-innovate name1 name2)
  (innovate name1 name2)
)

; — Rule 5: harden — add safety to a handler (high anomaly triggers hardening) —
(define (try-harden name)
  (harden name)
)

; — Rule 6: prune — remove a low-value handler (cycle even / overgrown) —
(define (try-prune name)
  (prune name)
)

; — Composite: CI-state aware evolution cycle —
(define (evolve-cycle name)
  (if name
    (begin
      ; High anomaly → repair + harden only
      (if (anomaly> 0.7) (begin (try-repair name) (try-harden name)) nil)

      ; Moderate anomaly → explore new handlers
      (if (and (anomaly> 0.3) (even-cycle?)) (try-explore name) nil)

      ; Many handlers + even cycle → prune for efficiency
      (if (and (many-handlers? 80) (even-cycle?)) (try-prune name) nil)

      ; Stable + odd cycle → exploit existing
      (if (not (anomaly> 0.7)) (try-exploit name) nil)
    )
    "evolve:no-target"
  )
)

;; ── S1-DeepResearch: dispatch condition example ──
;; Define should-skip to control handler dispatch from .ne files
;; When this returns true, the handler is skipped at dispatch time.
(define (should-skip name)
  ;; Skip bridge handler (it's a stub)
  (if (eq name "bridge") true
    ;; Skip signal pattern (also a stub)
    (if (eq name "signal_pattern") true
      false))))
