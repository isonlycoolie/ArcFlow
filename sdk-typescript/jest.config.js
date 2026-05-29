/** @type {import('jest').Config} */
module.exports = {
  preset: "ts-jest",
  testEnvironment: "node",
  roots: ["<rootDir>/tests"],
  testMatch: ["**/*.test.ts"],
  moduleNameMapper: {
    "^\\.\\./index\\.native\\.js$": "<rootDir>/tests/fixtures/mock-native.cjs",
  },
};
