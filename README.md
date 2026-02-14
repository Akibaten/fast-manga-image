# FMI - A fast image format for manga and comics.

The FMI (fast manga image) format provides fast, lossless compression of gray scale images at speeds of around 10 times faster decoding and 6-10 times faster encoding than PNG and with comparable sizes (worse in images with very few gradients and better in images with lots of gradients). It is optimized for use in already gray scale formats like manga and other comics.
The benchmark for conversion of an entire manga volume to .png vs to .fmi can be found below.

The recommended file extension for FMI is .fmi although this is not registered with MIME or reserved as a format. However, .fmi is not yet reserved in MIME by any format so collision with other files is unlikely.

## How it works
FMI uses an extremely simple format based on the concepts of the [QOI](https://qoiformat.org/) format. It encodes and decodes byte-wise with a single run across a file.
Much like [QOI](https://qoiformat.org/), a FMI file starts with a header followed by raw binary data is compressed into chunks of either one byte or two bytes.
There are four types of chunks total, and the types and specifications for these chunks can be found [here]().

To demonstrate the speed of FMI decoding, I have created a demo to show it rendering video in real time from a directory filled with frames of a video in .fmi format without an actual format. Each frame is read from memory as an image, decoded, and then painted to the viewer individually. This is at 60 frames per second, but given its speed it can go several times faster than this without dropping frames.

![](fmi_demo.gif)

## Limitations
FMI is gray scale by design, and while it performs similarly to PNG in size (often slightly worse). It has the advantage in massive gains for speed of decoding, and it was made with this in mind.

## Usage
The tool for encoding and decoding FMI files is written entirely in Rust. Usage and install guide below:

### Clone:
```
```
git clone https://github.com/Akibaten/fast-manga-image
cd fast-manga-image
```
```

### Build:
```
```
cargo install --path .
```
```

### Run:
__Encoding/Converting__
Encoding image files to FMI is capable with any format supported by the Rust Image crate:
```
fmi encode path/to/file_to_be_encoded
```
or
```
fmi convert path/to/file_to_be_converted
```
This will generate an FMI image with the same name and path as the input file
```
```
Alternatively a name for an output file can be specified.
```
fmi encode path/to/input_file output_file
```

__Decoding/Viewing__
Decoding images  will open the file in the built in image viewer built on the minifb Rust crate. The image viewer can be exited anytime by pressing escape.
```
fmi decode path/to/file_to_be_decoded
```
or
```
fmi view path/to/file_to_view
```

__Video Decoding__
For demonstration of FMI's speed, I have also included the ability to decode folders of .fmi frames as video.
This can be used like this.
```
fmi video path/to/directory_containing_frames
