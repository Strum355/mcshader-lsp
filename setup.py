import urllib.request
import zipfile
from io import BytesIO
import os as o

os = {
    0: 'win',
    1: 'linux',
    2: 'osx'
}

url = {
    0: 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-windows-x64-Release.zip',
    1: 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-linux-Release.zip',
    2: 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-osx-Release.zip'
}

def main():
    os_choice = int(input('Choose your OS:\n - 0: Windows\n - 1: Linux\n - 2: OSX\n> '))
    if os_choice not in os:
        print('Invalid OS. Please only choose a value between 0 and 2')
        exit(1)
    print('Downloading...')
    with urllib.request.urlopen(url[os_choice]) as respone:
        with BytesIO(respone.read()) as zipped:
            with zipfile.ZipFile(zipped) as zip_file:
                zip_file.extract('bin/glslangValidator')
                o.rename('bin/glslangValidator', 'glslangValidator')
                o.rmdir('bin')
                print('glslangValidator downloaded. Add this line to your VSCode settings:\n"mcglsl.glslangValidatorPath": "' + o.getcwd() + '/' + 'glslangValidator"')
                return
    print('There was an error :(')

main()