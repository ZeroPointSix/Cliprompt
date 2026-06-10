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

/**
 * Treat the first prompt search as settled whether it succeeds or fails so the
 * launcher can show either results or the existing error state.
 *
 * @template T
 * @param {Promise<T>} initialDataPromise
 * @param {{ scheduleAfterInitialData: () => void }} gate
 * @returns {Promise<T>}
 */
export async function notifyWhenInitialDataSettles(initialDataPromise, gate) {
  try {
    return await initialDataPromise;
  } finally {
    gate.scheduleAfterInitialData();
  }
}
