/**
 * Creates a small gate that keeps the backend window hidden until the launcher
 * has mounted and the initial prompt search has had a chance to update UI state.
 *
 * @param {() => Promise<unknown>} notifyBackendReady
 * @param {(callback: () => void) => void} [schedule]
 * @param {(error: unknown) => void} [onError]
 */
export function createLauncherReadyGate(
  notifyBackendReady,
  schedule = (callback) => setTimeout(callback, 0),
  onError = () => {}
) {
  let mounted = false;
  let scheduled = false;
  let notified = false;

  return {
    markMounted() {
      mounted = true;
    },

    scheduleAfterInitialData() {
      if (!mounted || scheduled || notified) {
        return;
      }

      scheduled = true;
      schedule(() => {
        void notifyBackendReady()
          .then(() => {
            notified = true;
          })
          .catch((error) => {
            scheduled = false;
            onError(error);
          });
      });
    }
  };
}
