import zipfile, sys
sys.path.insert(0, 'scripts')
from apply_qualium_branding_omni import deoptimize_jar

data = deoptimize_jar(open('runtime/browser/omni.ja', 'rb').read())
z = zipfile.ZipFile(__import__('io').BytesIO(data))

p = z.read('chrome/browser/content/browser/browser-places.js').decode('utf-8', 'ignore')
places_target = """    if (!organizer || organizer.closed) {
      // No currently open places window, so open one with the specified mode.
      openDialog(
        "chrome://browser/content/places/places.xhtml",
        "",
        "chrome,toolbar=yes,dialog=no,resizable",
        item
      );
    } else {
      organizer.PlacesOrganizer.selectLeftPaneContainerByHierarchy(item);
      organizer.focus();
    }"""
print('places_target in p:', places_target in p)

u = z.read('chrome/browser/content/browser/utilityOverlay.js').decode('utf-8', 'ignore')
about_target = '  window.openDialog("chrome://browser/content/aboutDialog.xhtml", "", features);'
print('about_target in u:', about_target in u)
