# Document

---

## Page 1

to test the OCR pipeline’s ability to detect mul- tiple columns and preserve correct reading order. The text should flow from the top of the left col-

Two-Column

Abstract

This document uses a standard two-column layout

umn to the bottom, then continue at the top of the right column. Mathematical equations, lists, and

right column.

regular text are included

to test various content

types.

Introduction

Academic papers frequently use two-column lay- outs to maximize information density while main-

taining readability. This presents a challenge for OCR systems, which must correctly identify col- umn boundaries and extract text in the proper

extract

reading order.

The goal of this test document is to verify that the OCR pipeline can:

Detect that the page has multiple columns

Separate blocks into left and right columns

Read the left column from top to bottom

Then read the right column from top to bot-

Failure to handle column layout correctly results

in text that alternates between columns, produc- ing an unreadable output like: "line 1 left, line 1

ing an unreadable output like: "line 1 left, line 1 right, line 2 left, line 2 right" instead of the correct "line 1 left, line 2 left, ..., line 1 right, line 2 right."

Mathematical Content

Mathematical notation is common in scientific pa- pers. Inline math like E = mc? and a+ 6 = ^

pers. Inline math like E = mc? 800 0 + 3 = 7 should be recognized correctly within each column. Display equations should also be handled prop-

erly:

en 02 = Jr

Two-Column Layout Test Testing Column Detection and Reading Order

Test

DocStruct Project

March 6, 2026

Vx B= p10 (까스) ot

The Fourier transform is defined as:

fw) =

7006

Lists and Enumeration

Unordered lists in columns:

First item in left column

Second item with more text to test line wrap-

ping

Third item with inline math 2? + y? = r?

Fourth item

proper

Ordered lists should maintain numbering:

Initial step

Processing phase

Validation stage

Final output generation

Korean Text Test

Two-column layouts are also used in Korean aca- demic papers. This section tests Korean text pro-

demic papers. This section tests Korear cessing in a multi-column environment.

Matrix and Symbols

Matrices should be preserved:

a1 021 031

413 423 433

Common symbols:

Is: a, 8,7, 5,€, A, fl, o, 7, w. and mathematical oper

Greek letters 1 mathematical operators V.0.f,SIL«0.¥.3.

should be recognized:

---

## Page 2

Algorithms

Logical expressions are common:

(PAQ)

Set theory notation:

Long

section

ings.

Conclusion

all content types (text, math, Ki properly handled. The expected reading order is:

with

algorithm.

References

a two-column layout,

correctly:

ference 2024.

Author

2025.

Author

Symposium 2026.

and Logic

AUB={a|xeAVae B}

ANB={x|rxeAAxce B}

Paragraph Test

longer

This section contains longer paragraphs to test how well the OCR handles continuous text across

contains

multiple lines within a single column. The text should flow naturally without breaks or misorder-

The text

Column detection should ensure that this

paragraph stays in its own column and doesn’t get mixed with text from the adjacent column. Word spacing and line breaks are important fac-

tors. The system should preserve natural reading flow and not introduce artificial breaks or concate-

nate words incorrectly. Hyphenation at line ends, if present, should be handled appropriately.

This two-column test document provides a com-

prehensive check for column detection and read- ing order in the DocStruct OCR pipeline. Suc-

the DocStruct OCR pipeline.

cessful processing will produce output that reads naturally from left column to right column, with all content types (text, math, Korean text, lists)

All content from left column (top to bottom)

All content from right column (top to bottom)

Any deviation from this order indicates a prob-

or block

column detection

sorting

Academic papers typically end with references. In a two-column layout, references should also flow

Author A. "Paper Title in Left Column". Con-

"Another

Title". Journal

Paper

Author C. "Research Topic". Workshop 2023.

"Column

Methods'.

Detection


**Math Equation 52:**

$$
R\equiv(-1P\lor\lnot\vert Q\lor R)
$$


