/**
 * @typedef {(hotkey: string, handler: () => void) => Promise<void>} HotkeyRegister
 * @typedef {(hotkey: string) => Promise<void>} HotkeyUnregister
 */

/**
 * Registers the new hotkey before releasing the old one so failures don't drop access.
 * @param {{
 *   currentHotkey: string | null,
 *   nextHotkey: string,
 *   register: HotkeyRegister,
 *   unregister: HotkeyUnregister,
 *   handler: () => void
 * }} options
 * @returns {Promise<{ currentHotkey: string | null, error: unknown | null, didRegister: boolean, didUnregister: boolean, unregisterError?: unknown }>}
 */
export async function registerHotkeySafely({
  currentHotkey,
  nextHotkey,
  register,
  unregister,
  handler
}) {
  if (currentHotkey === nextHotkey) {
    return {
      currentHotkey,
      error: null,
      didRegister: false,
      didUnregister: false
    };
  }

  try {
    await register(nextHotkey, handler);
  } catch (error) {
    return {
      currentHotkey,
      error,
      didRegister: false,
      didUnregister: false
    };
  }

  if (currentHotkey) {
    let unregisterError = null;
    try {
      await unregister(currentHotkey);
    } catch (error) {
      unregisterError = error;
    }
    return {
      currentHotkey: nextHotkey,
      error: null,
      didRegister: true,
      didUnregister: unregisterError === null,
      unregisterError
    };
  }

  return {
    currentHotkey: nextHotkey,
    error: null,
    didRegister: true,
    didUnregister: false
  };
}
