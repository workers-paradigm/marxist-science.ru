import { IconPicture } from '@codexteam/icons';

function make(tag, classList = []) {
  let elem = document.createElement(tag);
  classList.forEach(c => elem.classList.add(c));
  return elem;
}

class Image {
  // static getter for title and icon
  static get toolbox() {
    return {
      title: 'Image',
      icon: IconPicture,
    };
  }

  static get pasteConfig() {
    return {
      tags: [
        {
          img: {
            src: true,
          },
        },
      ],
      files: {
        mimetypes: ['image/jpeg', 'image/png', 'image/webp'],
      },
    };
  }

  static get isReadOnlySupported() {
    return true;
  }

  #holder;
  #imageURL;
  #error = '';
  #caption;
  #readOnly;

  constructor({ data, readOnly }) {
    this.#holder = make('div', ['image-picker']);
    this.#imageURL = data.url || '/static/img/cross.jpg';
    this.#caption = data.caption || '';
    this.#readOnly = readOnly;
  }

  #uploadAndRenderFile(file) {
    if (!file) {
      this.#error = 'Please select a file';
      this.render();
      return;
    }
    if (file.size > 25165824) {
      this.#error = 'Uploaded file is too big!';
      this.render();
      return;
    }
    if (
      !(file && ['image/png', 'image/webp', 'image/jpeg'].includes(file.type))
    ) {
      this.#error = 'Please upload one png/webp/jpeg file';
      this.render();
      return;
    }

    const form = new FormData();
    form.append('file', file);
    fetch('/upload/one_file', { method: 'PUT', body: form })
      .then(response => {
        if (response.ok) {
          return response.json();
        } else {
          throw new Error(response.statusText);
        }
      })
      .then(fileRecord => {
        this.#imageURL =
          '/static/uploads/' + fileRecord.hash + '.' + fileRecord.ext;
        this.#error = '';
      })
      .catch(error => (this.#error = error))
      .finally(() => this.render());
  }

  #renderReadOnly() {
    this.#holder = make('figure', ['captioned-image']);
    const image = make('img');
    const caption = make('figcaption', ['caption']);
    image.src = this.#imageURL;
    caption.innerText = this.#caption;
    this.#holder.replaceChildren(image);
    if (this.#caption !== '') {
      this.#holder.append(caption);
    }
    return this.#holder;
  }

  // We need at least of two methods to create a Block Tool for Editor.js — render and save.
  // Render returns the root element of the block
  render() {
    if (this.#readOnly) {
      return this.#renderReadOnly();
    }
    const caption = make('input', ['caption']);
    const img = make('img');

    caption.placeholder = 'caption';
    caption.value = this.#caption;
    caption.type = 'text';
    caption.addEventListener('change', e => (this.#caption = e.target.value));

    img.src = this.#imageURL;
    img.addEventListener('click', () => {
      const input = make('input');
      input.type = 'file';
      input.accept = 'image/png, image/webp, image/jpeg';
      input.addEventListener('change', () => {
        const file = input.files[0];
        this.#uploadAndRenderFile(file);
      });
      input.click();
    });

    this.#holder.replaceChildren(img, caption);
    this.#holder.addEventListener('drop', e => {
      e.preventDefault();
      if (e.dataTransfer) {
        const file = e.dataTransfer.files[0];
        this.#uploadAndRenderFile(file);
      }
    });

    if (this.#error !== '') {
      const error = make('span', ['error']);
      error.innerHTML = 'Error: ' + this.#error;
      this.#holder.prepend(error);
    }

    return this.#holder;
  }

  // Save returns an object that is saved on the server and is passed to the constructor
  save(holder) {
    return {
      url: this.#imageURL,
      caption: this.#caption,
    };
  }

  validate(savedData) {
    if (!savedData.url) {
      return false;
    }
    if (savedData.url.includes('/static/img/cross.jpg')) {
      return false;
    }

    return true;
  }

  onPaste(event) {
    switch (event.type) {
      case 'tag':
        if (event.detail.data.src) {
          this.#imageURL = event.detail.data.src;
          this.render();
        } else {
          this.#error('Invalid image copied from another website!');
          this.render();
        }
        break;
      case 'file':
        this.#uploadAndRenderFile(event.detail.file);
        break;
    }
  }
}

export { Image as default };
