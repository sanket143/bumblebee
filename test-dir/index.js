import { call } from "./factory.js";

const functionFactory = {
  mainFunction: () => {
    call();
  },
  testFunction: () => {
    return new Promise((resolve) => {
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

main();
/**
 * References of call()?
 * - functionFactory.mainFunction();
 * - functionFactory.testFunction();
 * - <anon>
 * - main();
 */
