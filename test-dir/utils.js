// another call just to test conflicts
// how will we track imports?
// was the call reference from index.js from `utils.js` or `factory.js`
export const call = () => {
  // this is from utls.js
  console.log();
};

// also a recursive example
function a() {
  b();
}

function b() {
  a();
}

function c() {
  b();
}
