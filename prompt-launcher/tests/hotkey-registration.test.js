import test from "node:test";
import assert from "node:assert/strict";
import { registerHotkeySafely } from "../src/lib/hotkey-registration.js";

test("registerHotkeySafely keeps hotkey when unchanged", async () => {
  const events = [];
  const register = async () => {
    events.push("register");
  };
  const unregister = async () => {
    events.push("unregister");
  };
  const handler = () => {};

  const result = await registerHotkeySafely({
    currentHotkey: "Alt+Space",
    nextHotkey: "Alt+Space",
    register,
    unregister,
    handler
  });

  assert.equal(result.currentHotkey, "Alt+Space");
  assert.equal(result.error, null);
  assert.deepEqual(events, []);
});

test("registerHotkeySafely registers before unregistering old hotkey", async () => {
  const events = [];
  const register = async () => {
    events.push("register");
  };
  const unregister = async () => {
    events.push("unregister");
  };
  const handler = () => {};

  const result = await registerHotkeySafely({
    currentHotkey: "Alt+Space",
    nextHotkey: "Alt+P",
    register,
    unregister,
    handler
  });

  assert.equal(result.currentHotkey, "Alt+P");
  assert.equal(result.error, null);
  assert.deepEqual(events, ["register", "unregister"]);
});

test("registerHotkeySafely keeps old hotkey when register fails", async () => {
  const events = [];
  const register = async () => {
    events.push("register");
    throw new Error("conflict");
  };
  const unregister = async () => {
    events.push("unregister");
  };
  const handler = () => {};

  const result = await registerHotkeySafely({
    currentHotkey: "Alt+Space",
    nextHotkey: "Alt+P",
    register,
    unregister,
    handler
  });

  assert.equal(result.currentHotkey, "Alt+Space");
  assert.ok(result.error);
  assert.deepEqual(events, ["register"]);
});

test("registerHotkeySafely reports unregister failures without blocking", async () => {
  const events = [];
  const register = async () => {
    events.push("register");
  };
  const unregister = async () => {
    events.push("unregister");
    throw new Error("unregister failed");
  };
  const handler = () => {};

  const result = await registerHotkeySafely({
    currentHotkey: "Alt+Space",
    nextHotkey: "Alt+P",
    register,
    unregister,
    handler
  });

  assert.equal(result.currentHotkey, "Alt+P");
  assert.equal(result.error, null);
  assert.equal(result.didUnregister, false);
  assert.ok(result.unregisterError);
  assert.deepEqual(events, ["register", "unregister"]);
});
