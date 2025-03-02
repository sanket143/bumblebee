// test imports manually
import { call } from "./factory.js";

const functionFactory = {
  mainFunction: () => {
    call();
  },
  testFunction: () => {
    return new Promise((resolve, reject) => {
      call();
      resolve("1");
    });
  },
};

const main = () => {
  call();

  Object.values(functionFactory).forEach((fn) => {
    console.log(fn());
  });
};

(() => {
  // what will be this function
  call();
})();

const unrelatedScope = () => {};

main();
/**
 * References of call()?
 * - functionFactory.mainFunction();
 * - functionFactory.testFunction();
 * - <anon>
 * - main();
 */
