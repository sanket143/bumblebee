// test imports manually
import { call } from "./factory.js";
import { fellow } from "utils"

const { mainFunction } = {
  mainFunction: () => {
    call();
    fellow();
  },
  testFunction: () => {
    return new Promise((resolve, reject) => {
      call();
      resolve("1");
    });
  },
};

const main = () => {
  mainFunction();

  Object.values(functionFactory).forEach((fn) => {
    console.log(fn());
  });
};

function foo() {
  // inside foo
  main();
}

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
