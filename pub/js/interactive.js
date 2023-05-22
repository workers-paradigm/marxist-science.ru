// Example of a label that transforms into a textarea when clicked. Supposed to
// also create a button to submit the changes and transform it back into a flat
// label
let sectorManagers = Array.from(document.querySelector('.manage-menu'));
sectorManagers.forEach((element) => {
    let nameSpan = element.querySelector('.title');
    console.log(nameSpan);
    nameSpan.addEventListener('click', (clickEvent) => {
        let contents = nameSpan.innerHTML;
        let textarea = document.createElement('textarea');
        textarea.appendChild(document.createTextNode(contents));
        nameSpan.innerHTML = "";
        nameSpan.append(textarea);
    });
});
