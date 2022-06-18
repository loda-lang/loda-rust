import { EditorView, basicSetup } from "codemirror";
// import { EditorState } from '@codemirror/state'
//import {basicSetup} from "codemirror"
// import {EditorState, EditorView, basicSetup} from "@codemirror/basic-setup"
// import {EditorView} from "@codemirror/view"
// import {EditorView, keymap} from "@codemirror/view"
// import {indentWithTab} from "@codemirror/commands"
import {javascript} from "@codemirror/lang-javascript"

// import {EditorView, basicSetup} from "@codemirror/basic-setup"
// import {javascript} from "@codemirror/lang-javascript"

const doc = `if (true) {
  console.log("okay")
} else {
  console.log("oh no")
}
`;

// alert('init');

//let editor = 
/*new EditorView({
  doc: doc,
  extensions: [
    basicSetup,
    // keymap.of([indentWithTab]),
    javascript()
  ],
  // extensions: [
  //   basicSetup, 
  //   javascript()
  // ],
  // state: EditorState.create({
  // }),
  parent: document.querySelector("#editor")
}); */

// new EditorView({
//   state: EditorState.create({
//     doc: doc,
//     extensions: [
//       basicSetup,
//       javascript()
//     ]
//   }),
//   parent: document.querySelector("#editor")
// });

new EditorView({
  parent: document.querySelector("#editor"),
  extensions: [
    basicSetup,
    javascript()
  ]
});
