import { EditorView, basicSetup } from "codemirror";
import {javascript} from "@codemirror/lang-javascript"

const doc = `if (true) {
  console.log("okay")
} else {
  console.log("oh no")
}
`;

new EditorView({
  doc: doc,
  parent: document.querySelector("#editor"),
  extensions: [
    basicSetup,
    javascript()
  ]
});
