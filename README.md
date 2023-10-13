A particle simulator, this is a work in progress.

### To Do

* bug: since window is rectangle, area of interaction is an oval and not a circle
  * fix: aspect ratio? don't use gl's coords for state, but the window's instead?
* feat: render low/zero/high pressure areas
* optimisations
  * parallel iteration when updating state?
  * don't compare every particle with every other particle (On^2), use spatial lookup
  * compute this on the GPU
