package main

import (
	"fmt"
	"image"
	"image/gif"
	"image/jpeg"
	"image/png"
	"os"
	"strings"

	"golang.org/x/image/bmp"
	"golang.org/x/image/tiff"
)

func main() {
	if len(os.Args) != 4 {
		fmt.Println("Usage: convert <input file> <output file> <format>")
		return
	}

	inputFile := os.Args[1]
	outputFile := os.Args[2]
	format := strings.ToLower(os.Args[3])

	file, err := os.Open(inputFile)
	if err != nil {
		fmt.Printf("Error opening input file: %v\n", err)
		return
	}
	defer file.Close()

	img, formatName, err := image.Decode(file)
	if err != nil {
		fmt.Printf("Error decoding image: %v\n", err)
		return
	}
	fmt.Printf("Input image format: %s\n", formatName)

	outFile, err := os.Create(outputFile)
	if err != nil {
		fmt.Printf("Error creating output file: %v\n", err)
		return
	}
	defer outFile.Close()

	switch format {
	case "jpeg", "jpg":
		err = jpeg.Encode(outFile, img, nil)
	case "png":
		err = png.Encode(outFile, img)
	case "gif":
		err = gif.Encode(outFile, img, nil)
	case "bmp":
		err = bmp.Encode(outFile, img)
	case "tiff":
		err = tiff.Encode(outFile, img, nil)
	default:
		fmt.Printf("Unsupported output format: %s\n", format)
		return
	}

	if err != nil {
		fmt.Printf("Error encoding image: %v\n", err)
		return
	}

	fmt.Println("Image conversion successful!")
}

