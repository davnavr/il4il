import * as playground from "il4il-playground"
import * as codemirror from "codemirror"
import {il4il} from "./mode-il4il"

/**
 * @type {{ name: string; content: (tab: HTMLElement) => void }[]}
 */
const TABS = [
    {
        name: "IL4IL",
        content: (tab) => {
            const editor = new codemirror.EditorView({
                extensions: [ codemirror.basicSetup, il4il() ],
                parent: tab,
                doc: ".section metadata {\n  .name \"MyModule\";\n}\n",
            });

            const state = playground.Playground.new();
            const compile_button = document.getElementById("button-compile");

            compile_button.addEventListener("click", () => {
                let buffer = ""; // TODO: Avoid string duplication
                for (const line of editor.state.doc) {
                    buffer += line + '\n';
                }
                const errors = state.assemble(buffer);
            });
        },
    },
    {
        name: "Binary",
        content: (tab) => {

        }
    }
];

function initialize_tabs() {
    const tabs_nav = window.document.getElementById("tabs-nav");
    const tabs_div = window.document.getElementById("tabs-div");

    /**
     * @type {HTMLButtonElement}
     */
    let first_tab = null;

    /**
     * @type {{ tab: HTMLElement, button: HTMLButtonElement } | null}
     */
    let active_tab = null;

    TABS.forEach(tab => {
        const button = window.document.createElement("button");
        const section = window.document.createElement("section");

        if (first_tab === null) {
            first_tab = button;
        }

        tabs_nav.appendChild(button);
        tabs_div.appendChild(section);

        button.classList.add("tab-button");
        button.innerText = tab.name;

        section.classList.add("tab", "hidden");

        let initialized = false;

        button.addEventListener("click", () => {
            if (active_tab !== null) {
                active_tab.tab.classList.add("hidden");
                active_tab.button.disabled = false;
            }

            section.classList.remove("hidden");
            button.disabled = true;
            active_tab = { tab: section, button };

            if (!initialized) {
                initialized = true;
                tab.content(section);
            }
        });
    });

    first_tab.click();
}

initialize_tabs();
