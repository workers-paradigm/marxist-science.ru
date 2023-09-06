import EditorJS from '@editorjs/editorjs';
import Header from '@editorjs/header';
import List from '@editorjs/list';
import Quote from '@editorjs/quote';
import Image from './image.js';

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
  },
});

let articleId = parseInt(document.getElementById('article-id').value);

editor.isReady.then(() => {
  fetch('/articles/contents/' + encodeURIComponent(articleId), {
    method: 'GET',
  }).then(async response => {
    if (response.ok) {
      let json = JSON.parse(await response.json());
      editor.render(json);
    }
  });

  const save = async () => {
    const errorElement = document.getElementById('response-error');
    const contents = await editor.save();
    const coverURL =
      contents.blocks.find(obj => obj.type === 'image')?.data?.url ?? null;
    let savedResult = {
      id: parseInt(document.getElementById('article-id').value),
      title: document.getElementById('article-title').value,
      coverURL: coverURL,
      contents: contents,
    };
    fetch('/articles/save', {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(savedResult),
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
    if (e.ctrlKey && e.key === 's') {
      e.preventDefault();
      save();
    }
  });
});
