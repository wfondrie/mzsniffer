# :skunk: mzsniffer :nose:

Detect polymer contaminants in mass spectra.

## Introduction
Mzsniffer is a command line application to quickly detect common polymer contaminants in mass spectrometry experiments. 
It is pretty dumb - mzsniffer merely extracts the intensities for common polymer precursors from the MS1 spectra of one or more mzML files. 
What it lacks in sophistication, mzsniffer makes up for in speed :rocket:. 
It only takes a few seconds to analyze most mzML files!

By default, mzsniffer logs the percentage of the total ion current that each polymer comprises. 
However, more detailed information can be saved to either JSON or as a pickled Python object using stdout (see the examples below). 

Give it a try and let me know how it goes!

## Installation
I haven't uploaded binaries yet, so this will be updated soon.

## Usage

## Developing

## Attributions

The mzML parsing code in mzsniffer was directly adapted from [Sage](https://github.com/lazear/sage) by @lazear... dragons :dragon: and all :wink:

Most of the polymers were adapted from [EncyclopeDIA](https://bitbucket.org/searleb/encyclopedia/wiki/Home) by @briansearle. 
