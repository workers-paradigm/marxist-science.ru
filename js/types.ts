// Taken from codex-team github (they are taking too long to merge a simple fix)
// https://github.com/codex-team/editor.js/pull/2158

export type TagConfig = boolean | { [attr: string]: boolean | string };

export type SanitizerRule = TagConfig | ((el: Element) => TagConfig)

export interface SanitizerConfig {
  /**
   * Tag name and params not to be stripped off
   * @see {@link https://github.com/guardian/html-janitor}
   *
   * @example Save P tags
   * p: true
   *
   * @example Save A tags and do not strip HREF attribute
   * a: {
   *   href: true
   * }
   *
   * @example Save A tags with TARGET="_blank" attribute
   * a: function (aTag) {
   *   return aTag.target === '_black';
   * }
   *
   * @example Save U tags that are not empty
   * u: function(el){
   *   return el.textContent !== '';
   * }
   *
   * @example For blockquote with class 'indent' save CLASS and STYLE attributes
   *          Otherwise strip all attributes
   * blockquote: function(el) {
   *   if (el.classList.contains('indent')) {
   *     return { 'class': true, 'style': true };
   *   } else {
   *     return {};
   *   }
   * }
   */
  [key: string]: SanitizerRule;
}

export interface BaseTool {
  /**
   * Tool`s render method
   * For inline Tools returns inline toolbar button
   * For block Tools returns tool`s wrapper
   */
  render(): HTMLElement;
}

export interface InlineTool extends BaseTool {
  /**
   * Shortcut for Tool
   * @type {string}
   */
  shortcut?: string;

  /**
   * Method that accepts selected range and wrap it somehow
   * @param {Range} range - selection's range
   */
  surround(range: Range): void;

  /**
   * Get SelectionUtils and detect if Tool was applied
   * For example, after that Tool can highlight button or show some details
   * @param {Selection} selection - current Selection
   */
  checkState(selection: Selection): boolean;

  /**
   * Make additional element with actions
   * For example, input for the 'link' tool or textarea for the 'comment' tool
   */
  renderActions?(): HTMLElement;

  /**
   * Function called with Inline Toolbar closing
   * @deprecated 2020 10/02 - The new instance will be created each time the button is rendered. So clear is not needed.
   *                          Better to create the 'destroy' method in a future.
   */
  clear?(): void;
}
