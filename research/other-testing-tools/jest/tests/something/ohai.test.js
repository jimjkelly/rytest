const ohai = require('../../src/ohai');

test('says ohai to the given name', () => {
  expect(ohai("there")).toBe("ohai there!");
});