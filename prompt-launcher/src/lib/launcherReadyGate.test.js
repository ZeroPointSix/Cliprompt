import test from "node:test";
import assert from "node:assert/strict";
import { createLauncherReadyGate } from "./launcherReadyGate.js";

const flushMicrotasks = () => Promise.resolve();

test("does not notify the backend before the frontend is mounted", async () => {
  /** @type {string[]} */
  const calls = [];
  /** @type {Array<() => void>} */
  const scheduled = [];
  const gate = createLauncherReadyGate(
    async () => calls.push("frontend_ready"),
    (callback) => scheduled.push(callback)
  );

  gate.scheduleAfterInitialData();
  assert.equal(scheduled.length, 0);
  assert.deepEqual(calls, []);

  gate.markMounted();
  gate.scheduleAfterInitialData();
  assert.equal(scheduled.length, 1);

  scheduled[0]();
  await flushMicrotasks();
  assert.deepEqual(calls, ["frontend_ready"]);
});

test("coalesces repeated ready requests into one backend notification", async () => {
  /** @type {string[]} */
  const calls = [];
  /** @type {Array<() => void>} */
  const scheduled = [];
  const gate = createLauncherReadyGate(
    async () => calls.push("frontend_ready"),
    (callback) => scheduled.push(callback)
  );

  gate.markMounted();
  gate.scheduleAfterInitialData();
  gate.scheduleAfterInitialData();
  gate.scheduleAfterInitialData();

  assert.equal(scheduled.length, 1);
  scheduled[0]();
  await flushMicrotasks();

  gate.scheduleAfterInitialData();
  assert.equal(scheduled.length, 1);
  assert.deepEqual(calls, ["frontend_ready"]);
});

test("allows retry when the backend notification fails", async () => {
  /** @type {unknown[]} */
  const errors = [];
  /** @type {Array<() => void>} */
  const scheduled = [];
  let attempts = 0;
  const gate = createLauncherReadyGate(
    async () => {
      attempts += 1;
      if (attempts === 1) {
        throw new Error("temporary failure");
      }
    },
    (callback) => scheduled.push(callback),
    (error) => errors.push(error)
  );

  gate.markMounted();
  gate.scheduleAfterInitialData();
  scheduled[0]();
  await flushMicrotasks();

  assert.equal(attempts, 1);
  assert.equal(errors.length, 1);

  gate.scheduleAfterInitialData();
  assert.equal(scheduled.length, 2);
  scheduled[1]();
  await flushMicrotasks();

  assert.equal(attempts, 2);
  assert.equal(errors.length, 1);
});
