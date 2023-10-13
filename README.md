A particle simulator, this is a work in progress.

### To Do

* feat: render low/zero/high pressure areas
* optimisations
  * parallel iteration when updating state?
  * don't compare every particle with every other particle (On^2), use spatial lookup
  * compute this on the GPU
* dev: be able to step through ticks
