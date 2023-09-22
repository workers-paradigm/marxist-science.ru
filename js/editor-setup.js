import EditorJS from '@editorjs/editorjs';
import Header from '@editorjs/header';
import List from '@editorjs/list';
import Quote from '@editorjs/quote';
import Separator from '@editorjs/delimiter';
import Image from './image.js';
import ItalicInlineTool from './inline-tool-italic.ts';
import BoldInlineTool from './inline-tool-bold.ts';

const editor = new EditorJS({
  holder: 'editorjs',
  minHeight: 20,
  tools: {
    heading: Header,
    list: {
      class: List,
      inlineToolbar: true,
    },
    quote: {
      class: Quote,
      inlineToolbar: true,
    },
    image: {
      class: Image,
    },
    bold: {
      class: BoldInlineTool,
    },
    italic: {
      class: ItalicInlineTool,
    },
    separator: Separator,
  },
});

const id = parseInt(document.getElementById('id').value);

editor.isReady.then(() => {
  fetch('/articles/contents/' + encodeURIComponent(id))
    .then(async response => {
      if (response.ok) {
        return response.json();
      } else {
        throw new Error('Request for contents failed, reason: ');
      }
    })
    .then(json => editor.render(JSON.parse(json)))
    .then(() => {
      const save = async () => {
        const errorElement = document.getElementById('response-error');
        const contents = await editor.save();
        fetch('/articles/save?id=' + id, {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(contents),
        })
          .then(async response => {
            // this handles all responses
            errorElement.innerText = response.ok ? '' : 'Some shitty error';
          })
          .catch(error => {
            // this handles erros from .then(...)
            errorElement.innerText =
              'Some shitty error, but another kind: ' + error;
          });
      };

      const saveButton = document.getElementById('save');
      saveButton.addEventListener('click', save);
      document.addEventListener('keydown', e => {
        if (e.ctrlKey && e.code === 'KeyS') {
          e.preventDefault();
          save();
        }
      });
    });
});
