// Temporary script to cycle through logo options
(function() {
  // Wait for the DOM to be fully loaded
  window.addEventListener('DOMContentLoaded', function() {
    // Get the logos ready
    const logoOptions = [
      '/img/logo-option-1.svg',
      '/img/logo-option-2.svg',
      '/img/logo-option-3.svg',
      '/img/logo-refined.svg',
      '/img/logo-final.svg',
      '/img/logo-oxidize-rb.svg'
    ];
    
    let currentLogoIndex = 0;
    
    // Function to update the logo
    function cycleLogo() {
      // Find the logo container
      const logoContainer = document.querySelector('.logo-container');
      const logoImg = document.querySelector('.navbar__logo img');
      const svgLogo = document.querySelector('.logo-container svg');

      if (logoImg) {
        // If using an img tag
        logoImg.src = logoOptions[currentLogoIndex];
        logoImg.style.transition = 'opacity 0.3s ease';
        logoImg.style.opacity = 0;

        setTimeout(() => {
          logoImg.style.opacity = 1;
        }, 50);
      } else if (logoContainer) {
        // If using our custom container with SVG
        const newImg = document.createElement('img');
        newImg.src = logoOptions[currentLogoIndex];
        newImg.alt = 'oxidize.rb Logo';
        newImg.style.height = '32px';
        newImg.style.width = '32px';

        // Clear container and add the new image
        logoContainer.innerHTML = '';
        logoContainer.appendChild(newImg);
      } else if (svgLogo) {
        // If using an inline SVG directly
        const newImg = document.createElement('img');
        newImg.src = logoOptions[currentLogoIndex];
        newImg.alt = 'oxidize.rb Logo';
        newImg.className = svgLogo.className;
        newImg.style.height = '32px';
        newImg.style.width = '32px';

        // Replace the SVG with the img
        svgLogo.parentNode.replaceChild(newImg, svgLogo);
      }
      
      // Update index for next cycle
      currentLogoIndex = (currentLogoIndex + 1) % logoOptions.length;
    }
    
    // Check and run the cycle every second
    setInterval(cycleLogo, 1000);
    
    // Run it immediately to start
    cycleLogo();
    
    // Add a note to the page about the cycling logos
    const noteDiv = document.createElement('div');
    noteDiv.style.position = 'fixed';
    noteDiv.style.bottom = '10px';
    noteDiv.style.right = '10px';
    noteDiv.style.background = 'rgba(0,0,0,0.7)';
    noteDiv.style.color = 'white';
    noteDiv.style.padding = '8px';
    noteDiv.style.borderRadius = '4px';
    noteDiv.style.fontSize = '12px';
    noteDiv.style.zIndex = '9999';
    noteDiv.innerHTML = 'Cycling through logo options... <button id="stop-cycling" style="margin-left: 8px; padding: 2px 6px;">Stop</button>';
    
    document.body.appendChild(noteDiv);
    
    // Store the interval for potential stopping
    const cycleInterval = setInterval(cycleLogo, 1000);

    // Add stop functionality
    document.getElementById('stop-cycling').addEventListener('click', function() {
      clearInterval(cycleInterval);
      noteDiv.innerHTML = 'Logo cycling stopped. Current: Option ' + (currentLogoIndex === 0 ? 3 : currentLogoIndex);
      setTimeout(() => {
        noteDiv.style.opacity = 0;
        noteDiv.style.transition = 'opacity 0.5s ease';
      }, 2000);
      setTimeout(() => noteDiv.remove(), 2500);
    });
  });
})();