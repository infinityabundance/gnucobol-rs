# Diagnostic-unblocked — semantic reachability

_schema: gnurust-diag-unblocked-reachability-v1 · transformer gnurust-diag-unblocked-transform-v1 · source stable-3.2

The primary question is NOT “more tests passed”. It is: which later semantic
checks became reachable solely because compiler diagnostic text stopped gating
the group. Ignored diagnostic text is NOT diagnostic compatibility.

## Totals

| diagnostic_expectations_ignored | 621 |
| stdout_ignored | 1 |
| stderr_ignored | 620 |
| groups_affected | 404 |
| groups_affected_not_in_suite | 0 |
| groups_analyzed_with_check_detail | 948 |
| groups_progressed_further | 377 |
| groups_no_additional_step | 27 |
| gate_lifted_no_progress | 1 |
| groups_later_compile_reached | 111 |
| groups_execution_reached | 27 |
| newly_reached_checks | 140 |
| newly_reached_runtime_checks | 27 |
| newly_matched_runtime_checks | 12 |
| newly_exposed_compile_failures | 17 |
| newly_exposed_runtime_failures | 15 |
| newly_exposed_artifact_failures | 1 |
| pristine_group_passes | 196 |
| unblocked_group_passes | 326 |
| pristine_candidate_xpass | 0 |
| unblocked_candidate_xpass | 2 |
| suite_groups | 1282 |

## Oracle cross-reference

pristine oracle XPASS: 0 · unblocked oracle XPASS: 4

- group 116 `OSVS register CURRENT-DATE and TIME-OF-DAY` — syn_definition.at — pristine: no trace — xfail: true
- group 323 `checking prototypes and definitions match` — syn_misc.at — pristine: no trace — xfail: true
- group 336 `USE FOR DEBUGGING syntax-checks (2)` — syn_misc.at — pristine: no trace — xfail: true
- group 350 `Fall-Through to WHEN OTHER` — syn_misc.at — pristine: no trace — xfail: true

## Groups that progressed further

| group | file | title | ignored | pristine stop | unblocked stop | newly reached |
|---|---|---|---|---|---|---|
| 2 | used_binaries.at | compiler warnings | 15 | 1 TEXT_ONLY | all passed | 4 |
| 7 | used_binaries.at | source file not found | 1 | 0 TEXT_ONLY | all passed | 0 |
| 8 | used_binaries.at | temporary path invalid | 2 | 0 TEXT_ONLY | all passed | 2 |
| 11 | used_binaries.at | invalid cobc option | 2 | 0 TEXT_ONLY | all passed | 1 |
| 19 | used_binaries.at | run job with unsuccessful compilation | 1 | 0 TEXT_ONLY | all passed | 0 |
| 26 | configuration.at | cobc with standard configuration file | 1 | 0 TEXT_ONLY | all passed | 0 |
| 27 | configuration.at | cobc dialect features for all -std | 12 | 0 TEXT_ONLY | all passed | 2 |
| 28 | configuration.at | cobc with configuration file via -std | 1 | 0 TEXT_ONLY | all passed | 0 |
| 29 | configuration.at | cobc with standard configuration file via -conf | 1 | 0 TEXT_ONLY | all passed | 0 |
| 31 | configuration.at | cobc configuration: recursive include | 1 | 0 TEXT_ONLY | all passed | 0 |
| 32 | configuration.at | cobc with -std and -conf | 1 | 0 TEXT_ONLY | all passed | 0 |
| 35 | configuration.at | cobc configuration: entries | 4 | 0 TEXT_ONLY | 2 | 2 |
| 36 | configuration.at | cobc configuration: conf missing | 3 | 0 TEXT_ONLY | all passed | 2 |
| 37 | configuration.at | cobc configuration: conf optional | 1 | 0 TEXT_ONLY | all passed | 1 |
| 48 | configuration.at | cobc configuration: source format | 3 | 0 TEXT_ONLY | 3 | 3 |
| 54 | syn_copy.at | COPY: file not found | 4 | 0 TEXT_ONLY | all passed | 3 |
| 67 | syn_copy.at | COPY: multiple partial matches with an error | 1 | 0 TEXT_ONLY | all passed | 0 |
| 71 | syn_copy.at | COPY and REPLACE errors | 1 | 0 TEXT_ONLY | all passed | 0 |
| 73 | syn_definition.at | Invalid source name | 1 | 0 TEXT_ONLY | all passed | 0 |
| 74 | syn_definition.at | Invalid PROGRAM-ID | 3 | 0 TEXT_ONLY | all passed | 0 |
| 75 | syn_definition.at | Invalid PROGRAM-ID type clause (1) | 1 | 0 TEXT_ONLY | all passed | 0 |
| 76 | syn_definition.at | invalid PROGRAM-ID type clause (2) | 1 | 0 TEXT_ONLY | all passed | 0 |
| 78 | syn_definition.at | Undefined data name | 1 | 0 TEXT_ONLY | all passed | 0 |
| 79 | syn_definition.at | Undefined group name | 1 | 0 TEXT_ONLY | all passed | 0 |
| 80 | syn_definition.at | Undefined data name in group | 1 | 0 TEXT_ONLY | all passed | 0 |
| 81 | syn_definition.at | Reference not a group name | 1 | 0 TEXT_ONLY | all passed | 0 |
| 82 | syn_definition.at | Incomplete 01 definition | 1 | 0 TEXT_ONLY | all passed | 0 |
| 83 | syn_definition.at | error handling in conditions | 1 | 0 TEXT_ONLY | all passed | 0 |
| 84 | syn_definition.at | Same paragraphs in different sections | 1 | 1 TEXT_ONLY | all passed | 0 |
| 85 | syn_definition.at | GO TO sections and foreign paragraphs | 2 | 0 TEXT_ONLY | all passed | 1 |
| 87 | syn_definition.at | Redefinition of 01 and 02 items | 1 | 0 TEXT_ONLY | all passed | 0 |
| 88 | syn_definition.at | Redefinition of 02 items | 1 | 0 TEXT_ONLY | all passed | 0 |
| 89 | syn_definition.at | Redefinition of 77 items | 1 | 0 TEXT_ONLY | all passed | 0 |
| 90 | syn_definition.at | Redefinition of 01 and 77 items | 1 | 0 TEXT_ONLY | all passed | 0 |
| 91 | syn_definition.at | Redefinition of 88 items | 1 | 0 TEXT_ONLY | all passed | 0 |
| 93 | syn_definition.at | Redefinition of program-name within program | 2 | 0 TEXT_ONLY | all passed | 0 |
| 94 | syn_definition.at | Redefinition of function-prototype name | 1 | 0 TEXT_ONLY | all passed | 0 |
| 95 | syn_definition.at | PROCEDURE DIVISION RETURNING OMITTED: main | 1 | 1 TEXT_ONLY | all passed | 0 |
| 96 | syn_definition.at | PROCEDURE DIVISION RETURNING OMITTED: FUNCTION | 1 | 0 TEXT_ONLY | all passed | 0 |
| 99 | syn_definition.at | Ambiguous reference to 02 items | 1 | 0 TEXT_ONLY | all passed | 0 |
| 100 | syn_definition.at | Ambiguous reference to 02 and 03 items | 1 | 0 TEXT_ONLY | all passed | 0 |
| 101 | syn_definition.at | Ambiguous reference with qualification | 1 | 0 TEXT_ONLY | all passed | 0 |
| 103 | syn_definition.at | SYNCHRONIZED clause | 1 | 1 TEXT_ONLY | all passed | 0 |
| 104 | syn_definition.at | Undefined procedure name | 1 | 0 TEXT_ONLY | all passed | 0 |
| 105 | syn_definition.at | Redefinition of section names | 1 | 0 TEXT_ONLY | all passed | 0 |
| 106 | syn_definition.at | Redefinition of section and paragraph names | 1 | 0 TEXT_ONLY | all passed | 0 |
| 107 | syn_definition.at | Redefinition of label and variable names | 1 | 0 TEXT_ONLY | 1 | 1 |
| 109 | syn_definition.at | Ambiguous reference to paragraph name | 1 | 0 TEXT_ONLY | all passed | 0 |
| 110 | syn_definition.at | Non-matching level numbers (extension) | 2 | 0 TEXT_ONLY | all passed | 0 |
| 111 | syn_definition.at | CALL BY VALUE alphanumeric item (extension) | 1 | 0 TEXT_ONLY | all passed | 0 |
| 112 | syn_definition.at | CALL BY VALUE national item (extension) | 1 | 0 TEXT_ONLY | all passed | 0 |
| 114 | syn_definition.at | Duplicate identification division header | 1 | 0 TEXT_ONLY | all passed | 0 |
| 115 | syn_definition.at | RETURNING in STOP RUN / GOBACK / EXIT PROGRAM | 1 | 1 TEXT_ONLY | all passed | 0 |
| 116 | syn_definition.at | OSVS register CURRENT-DATE and TIME-OF-DAY | 5 | 0 TEXT_ONLY | all passed | 0 |
| 117 | syn_definition.at | Invalid ENVIRONMENT DIVISION order | 1 | 0 TEXT_ONLY | all passed | 0 |
| 118 | syn_definition.at | Function without END FUNCTION | 1 | 0 TEXT_ONLY | all passed | 0 |
| 120 | syn_definition.at | Nested programs not in procedure division | 1 | 0 TEXT_ONLY | all passed | 0 |
| 122 | syn_definition.at | Invalid PICTURE strings | 2 | 0 TEXT_ONLY | all passed | 1 |
| 123 | syn_definition.at | PICTURE string with control character | 1 | 0 TEXT_ONLY | all passed | 0 |
| 124 | syn_definition.at | PICTURE strings invalid with BLANK WHEN ZERO | 1 | 0 TEXT_ONLY | all passed | 0 |
| 125 | syn_definition.at | PICTURE strings invalid with USAGE | 1 | 0 TEXT_ONLY | all passed | 0 |
| 127 | syn_definition.at | ALPHABET definition | 1 | 0 TEXT_ONLY | all passed | 0 |
| 128 | syn_definition.at | PROGRAM COLLATING SEQUENCE | 3 | 0 TEXT_ONLY | all passed | 2 |
| 129 | syn_definition.at | RENAMES item | 1 | 0 TEXT_ONLY | all passed | 0 |
| 130 | syn_definition.at | RENAMES of 01-, 66- and 77-level items | 1 | 0 TEXT_ONLY | 1 | 1 |
| 131 | syn_definition.at | SAME AS clause | 2 | 1 TEXT_ONLY | all passed | 0 |
| 133 | syn_definition.at | LIKE clause | 1 | 1 TEXT_ONLY | all passed | 0 |
| 134 | syn_definition.at | APPLY COMMIT clause | 1 | 0 TEXT_ONLY | all passed | 0 |
| 135 | syn_definition.at | GLOBAL record-names | 1 | 0 TEXT_ONLY | all passed | 0 |
| 136 | syn_definition.at | Invalid USE BEFORE | 1 | 0 TEXT_ONLY | all passed | 0 |
| 137 | syn_subscripts.at | Non-numeric subscript | 1 | 0 TEXT_ONLY | all passed | 0 |
| 138 | syn_subscripts.at | Subscript range check | 2 | 0 TEXT_ONLY | all passed | 0 |
| 139 | syn_subscripts.at | Subscript bounds with OCCURS DEPENDING ON | 1 | 0 TEXT_ONLY | all passed | 0 |
| 140 | syn_subscripts.at | Subscripted item requires OCCURS clause | 1 | 0 TEXT_ONLY | all passed | 0 |
| 141 | syn_subscripts.at | Number of subscripts | 2 | 0 TEXT_ONLY | all passed | 0 |
| 142 | syn_subscripts.at | SET SSRANGE syntax | 1 | 0 TEXT_ONLY | all passed | 0 |
| 143 | syn_occurs.at | OCCURS with level 01 and 77 | 2 | 0 TEXT_ONLY | 1 | 1 |
| 144 | syn_occurs.at | OCCURS with level 66 | 1 | 0 TEXT_ONLY | all passed | 0 |
| 145 | syn_occurs.at | OCCURS with level 78 | 1 | 0 TEXT_ONLY | all passed | 0 |
| 146 | syn_occurs.at | OCCURS with level 88 | 1 | 0 TEXT_ONLY | all passed | 0 |
| 147 | syn_occurs.at | OCCURS with variable-occurrence data item | 6 | 0 TEXT_ONLY | 1 | 1 |
| 149 | syn_occurs.at | OCCURS data-items for INDEXED and KEY | 1 | 0 TEXT_ONLY | all passed | 0 |
| 151 | syn_occurs.at | OCCURS DEPENDING with wrong size | 1 | 0 TEXT_ONLY | all passed | 0 |
| 152 | syn_occurs.at | OCCURS DEPENDING followed by another field | 1 | 0 TEXT_ONLY | 1 | 1 |
| 153 | syn_occurs.at | OCCURS with unmatched DEPENDING / TO phrases | 3 | 0 TEXT_ONLY | all passed | 0 |
| 154 | syn_occurs.at | OCCURS INDEXED before KEY | 2 | 0 TEXT_ONLY | all passed | 1 |
| 156 | syn_redefines.at | REDEFINES: not following entry-name | 2 | 0 TEXT_ONLY | all passed | 1 |
| 157 | syn_redefines.at | REDEFINES: level 02 by 01 | 1 | 0 TEXT_ONLY | all passed | 0 |
| 158 | syn_redefines.at | REDEFINES: level 03 by 02 | 1 | 0 TEXT_ONLY | all passed | 0 |
| 159 | syn_redefines.at | REDEFINES: level 66 | 1 | 0 TEXT_ONLY | all passed | 0 |
| 160 | syn_redefines.at | REDEFINES: level 88 | 1 | 0 TEXT_ONLY | all passed | 0 |
| 161 | syn_redefines.at | REDEFINES: lower level number | 1 | 0 TEXT_ONLY | all passed | 0 |
| 162 | syn_redefines.at | REDEFINES: with OCCURS | 1 | 0 TEXT_ONLY | all passed | 0 |
| 163 | syn_redefines.at | REDEFINES: with subscript | 1 | 0 TEXT_ONLY | all passed | 0 |
| 164 | syn_redefines.at | REDEFINES: with variable occurrence | 1 | 0 TEXT_ONLY | all passed | 0 |
| 165 | syn_redefines.at | REDEFINES: with qualification | 1 | 0 TEXT_ONLY | all passed | 0 |
| 166 | syn_redefines.at | REDEFINES: multiple redefinition | 1 | 0 TEXT_ONLY | all passed | 0 |
| 168 | syn_redefines.at | REDEFINES: with VALUE | 1 | 0 TEXT_ONLY | 1 | 1 |
| 169 | syn_redefines.at | REDEFINES: with intervention | 1 | 0 TEXT_ONLY | all passed | 0 |
| 171 | syn_redefines.at | REDEFINES: for ANY LENGTH item | 1 | 0 TEXT_ONLY | all passed | 0 |
| 172 | syn_redefines.at | REDEFINES: non-referenced ambiguous item | 1 | 0 TEXT_ONLY | all passed | 0 |
| 173 | syn_value.at | bad VALUES / VALUES ARE in format-1 | 2 | 0 TEXT_ONLY | all passed | 1 |
| 174 | syn_value.at | OCCURS too many VALUEs | 1 | 0 TEXT_ONLY | all passed | 0 |
| 175 | syn_value.at | Numeric item (integer) | 1 | 0 TEXT_ONLY | all passed | 0 |
| 176 | syn_value.at | Numeric item (non-integer) | 1 | 0 TEXT_ONLY | all passed | 0 |
| 177 | syn_value.at | Numeric item with picture P | 1 | 0 TEXT_ONLY | all passed | 0 |
| 178 | syn_value.at | Signed numeric literal | 1 | 0 TEXT_ONLY | all passed | 0 |
| 179 | syn_value.at | Alphabetic item | 1 | 0 TEXT_ONLY | all passed | 0 |
| 180 | syn_value.at | Alphanumeric item | 1 | 0 TEXT_ONLY | all passed | 0 |
| 181 | syn_value.at | Alphanumeric group item | 1 | 0 TEXT_ONLY | all passed | 0 |
| 182 | syn_value.at | National item | 1 | 0 TEXT_ONLY | all passed | 0 |
| 183 | syn_value.at | Numeric-edited item | 3 | 0 TEXT_ONLY | all passed | 1 |
| 184 | syn_value.at | Alphanumeric-edited item | 1 | 0 TEXT_ONLY | all passed | 0 |
| 185 | syn_value.at | Implicit picture from value | 1 | 0 TEXT_ONLY | all passed | 0 |
| 186 | syn_file.at | Missing SELECT | 1 | 0 TEXT_ONLY | all passed | 0 |
| 187 | syn_file.at | Duplicated SELECT | 1 | 0 TEXT_ONLY | all passed | 0 |
| 188 | syn_file.at | Missing FD | 1 | 0 TEXT_ONLY | all passed | 0 |
| 189 | syn_file.at | Duplicated FD | 1 | 0 TEXT_ONLY | all passed | 0 |
| 193 | syn_file.at | ASSIGN to variable | 2 | 0 TEXT_ONLY | all passed | 0 |
| 194 | syn_file.at | SELECT without ASSIGN | 1 | 0 TEXT_ONLY | all passed | 0 |
| 195 | syn_file.at | START on SEQUENTIAL file | 1 | 0 TEXT_ONLY | all passed | 0 |
| 196 | syn_file.at | OPEN SEQUENTIAL file REVERSED | 2 | 0 TEXT_ONLY | all passed | 1 |
| 197 | syn_file.at | OPEN SEQUENTIAL file NO REWIND | 1 | 0 TEXT_ONLY | all passed | 0 |
| 199 | syn_file.at | INDEXED file invalid key items | 1 | 0 TEXT_ONLY | all passed | 0 |
| 200 | syn_file.at | variable record length | 5 | 0 TEXT_ONLY | all passed | 0 |
| 201 | syn_file.at | variable record length DEPENDING item | 4 | 0 TEXT_ONLY | all passed | 0 |
| 203 | syn_file.at | DECLARATIVES invalid procedure reference (2) | 1 | 0 TEXT_ONLY | all passed | 0 |
| 205 | syn_file.at | RECORDING MODE | 1 | 0 TEXT_ONLY | all passed | 0 |
| 206 | syn_file.at | CODE-SET clause | 1 | 0 TEXT_ONLY | all passed | 0 |
| 207 | syn_file.at | CODE-SET FOR clause | 1 | 0 TEXT_ONLY | all passed | 0 |
| 208 | syn_file.at | WRITE / REWRITE FROM clause and FILE | 1 | 0 TEXT_ONLY | all passed | 0 |
| 209 | syn_file.at | Clauses following invalid ACCESS clause | 1 | 0 TEXT_ONLY | all passed | 0 |
| 210 | syn_file.at | RELATIVE KEY type validation | 1 | 0 TEXT_ONLY | all passed | 0 |
| 211 | syn_file.at | Mismatched KEY clause | 1 | 0 TEXT_ONLY | all passed | 0 |
| 212 | syn_file.at | RECORD DELIMITER | 1 | 0 TEXT_ONLY | all passed | 0 |
| 213 | syn_file.at | FILE STATUS | 1 | 0 TEXT_ONLY | all passed | 0 |
| 214 | syn_file.at | VSAM status | 2 | 1 TEXT_ONLY | all passed | 1 |
| 215 | syn_file.at | INDEXED file PASSWORD clause | 1 | 0 TEXT_ONLY | all passed | 0 |
| 216 | syn_file.at | RECORD clause equal limits | 1 | 0 TEXT_ONLY | all passed | 0 |
| 217 | syn_file.at | FILE ... FROM literal | 3 | 0 TEXT_ONLY | all passed | 0 |
| 218 | syn_file.at | WRITE / REWRITE on REPORT files | 1 | 0 TEXT_ONLY | all passed | 0 |
| 219 | syn_file.at | SELECT without fd-name | 1 | 0 TEXT_ONLY | all passed | 0 |
| 220 | syn_file.at | Undeclared FILE-ID variable | 1 | 0 TEXT_ONLY | all passed | 0 |
| 223 | syn_file.at | ACCESS RANDOM with ORG SEQUENTIAL | 1 | 0 TEXT_ONLY | all passed | 0 |
| 226 | syn_file.at | ALTERNATE RECORD definition WITH NO DUPLICATES | 1 | 0 TEXT_ONLY | 1 | 1 |
| 227 | syn_file.at | ALTERNATE RECORD definition omitting RECORD | 1 | 0 TEXT_ONLY | all passed | 0 |
| 228 | syn_file.at | SELECT/OPEN syntax extensions | 2 | 0 TEXT_ONLY | all passed | 1 |
| 230 | syn_file.at | Invalid file name in SELECT | 1 | 0 TEXT_ONLY | all passed | 0 |
| 231 | syn_reportwriter.at | REPORT error/warning | 1 | 0 TEXT_ONLY | all passed | 0 |
| 232 | syn_reportwriter.at | REPORT not positive integers in COL / LINE PLUS | 1 | 0 TEXT_ONLY | all passed | 0 |
| 233 | syn_reportwriter.at | Missing PICTURE for SOURCE | 1 | 0 TEXT_ONLY | all passed | 0 |
| 234 | syn_reportwriter.at | Missing DETAIL line | 1 | 0 TEXT_ONLY | all passed | 0 |
| 235 | syn_reportwriter.at | REPORT LINE PLUS ZERO | 1 | 0 TEXT_ONLY | all passed | 0 |
| 236 | syn_reportwriter.at | Incorrect REPORT NAME | 2 | 0 TEXT_ONLY | all passed | 1 |
| 237 | syn_reportwriter.at | REPORT with PLUS RIGHT/CENTER | 1 | 0 TEXT_ONLY | all passed | 0 |
| 238 | syn_reportwriter.at | PAGE LIMITS clause | 1 | 0 TEXT_ONLY | all passed | 0 |
| 239 | syn_reportwriter.at | Report FD without period | 2 | 0 TEXT_ONLY | all passed | 1 |
| 241 | syn_reportwriter.at | Incorrect USAGE clause | 1 | 0 TEXT_ONLY | all passed | 0 |
| 242 | syn_refmod.at | valid reference-modification | 1 | 0 TEXT_ONLY | all passed | 0 |
| 243 | syn_refmod.at | invalid reference-modification | 1 | 0 TEXT_ONLY | all passed | 0 |
| 244 | syn_refmod.at | Static out of bounds | 1 | 0 TEXT_ONLY | all passed | 0 |
| 245 | syn_refmod.at | constant-folding out of bounds | 1 | 0 TEXT_ONLY | all passed | 0 |
| 247 | syn_misc.at | ambiguous AND/OR | 1 | 0 TEXT_ONLY | all passed | 0 |
| 249 | syn_misc.at | warn literal size | 1 | 0 TEXT_ONLY | all passed | 0 |
| 250 | syn_misc.at | warn literal size in constant expr. (level 88) | 1 | 0 TEXT_ONLY | all passed | 0 |
| 251 | syn_misc.at | Invalid conditional expression (1) | 1 | 0 TEXT_ONLY | all passed | 0 |
| 252 | syn_misc.at | Invalid conditional expression (2) | 1 | 0 TEXT_ONLY | all passed | 0 |
| 253 | syn_misc.at | Invalid conditional expression (3) | 1 | 0 TEXT_ONLY | all passed | 0 |
| 255 | syn_misc.at | missing headers | 2 | 0 TEXT_ONLY | all passed | 1 |
| 256 | syn_misc.at | one line program | 2 | 0 TEXT_ONLY | all passed | 1 |
| 257 | syn_misc.at | empty program | 3 | 4 TEXT_ONLY | all passed | 1 |
| 258 | syn_misc.at | INITIALIZE constant | 1 | 0 TEXT_ONLY | all passed | 0 |
| 259 | syn_misc.at | CLASS duplicate values | 1 | 0 TEXT_ONLY | all passed | 0 |
| 260 | syn_misc.at | INSPECT invalid size | 1 | 0 TEXT_ONLY | all passed | 0 |
| 261 | syn_misc.at | INSPECT invalid target | 1 | 0 TEXT_ONLY | all passed | 0 |
| 262 | syn_misc.at | INSPECT missing keyword | 1 | 0 TEXT_ONLY | all passed | 0 |
| 263 | syn_misc.at | INSPECT repeated keywords | 1 | 0 TEXT_ONLY | all passed | 0 |
| 264 | syn_misc.at | INSPECT incomplete clause | 1 | 0 TEXT_ONLY | all passed | 0 |
| 265 | syn_misc.at | INSPECT multiple BEFORE/AFTER clauses | 1 | 0 TEXT_ONLY | all passed | 0 |
| 266 | syn_misc.at | EXAMINE invalid literals | 1 | 0 TEXT_ONLY | all passed | 0 |
| 267 | syn_misc.at | maximum data size | 1 | 1 TEXT_ONLY | all passed | 0 |
| 268 | syn_misc.at | unreachable statement | 1 | 0 TEXT_ONLY | all passed | 0 |
| 269 | syn_misc.at | CRT STATUS | 1 | 0 TEXT_ONLY | all passed | 0 |
| 270 | syn_misc.at | SPECIAL-NAMES clause | 3 | 0 TEXT_ONLY | all passed | 0 |
| 271 | syn_misc.at | CURRENCY SIGN | 6 | 0 TEXT_ONLY | all passed | 6 |
| 272 | syn_misc.at | SWITCHES | 2 | 0 TEXT_ONLY | all passed | 0 |
| 273 | syn_misc.at | unexpected mnemonic-name location | 1 | 0 TEXT_ONLY | all passed | 0 |
| 274 | syn_misc.at | wrong device for mnemonic-name | 1 | 0 TEXT_ONLY | all passed | 0 |
| 275 | syn_misc.at | missing mnemonic-name declaration | 1 | 0 TEXT_ONLY | all passed | 0 |
| 276 | syn_misc.at | unknown device in dialect | 1 | 1 TEXT_ONLY | all passed | 0 |
| 282 | syn_misc.at | source text after program-text area | 1 | 0 TEXT_ONLY | all passed | 0 |
| 283 | syn_misc.at | line overflow in fixed-form / free-form | 4 | 1 TEXT_ONLY | all passed | 3 |
| 284 | syn_misc.at | missing newline in fixed-form / free-form | 3 | 0 TEXT_ONLY | all passed | 3 |
| 285 | syn_misc.at | continuation of COBOL words | 2 | 0 TEXT_ONLY | all passed | 1 |
| 286 | syn_misc.at | line and floating comments | 7 | 2 TEXT_ONLY | 3 | 1 |
| 287 | syn_misc.at | word length | 6 | 0 TEXT_ONLY | all passed | 3 |
| 288 | syn_misc.at | Segmentation Module | 2 | 0 TEXT_ONLY | all passed | 1 |
| 291 | syn_misc.at | ACUCOBOL USAGE HANDLE | 2 | 0 TEXT_ONLY | all passed | 1 |
| 292 | syn_misc.at | ACUCOBOL WINDOW statements | 2 | 0 TEXT_ONLY | all passed | 0 |
| 293 | syn_misc.at | ACUCOBOL GRAPHICAL controls | 1 | 0 TEXT_ONLY | all passed | 0 |
| 294 | syn_misc.at | DISPLAY MESSAGE BOX | 1 | 0 TEXT_ONLY | all passed | 0 |
| 295 | syn_misc.at | DISPLAY OMITTED | 1 | 0 TEXT_ONLY | all passed | 0 |
| 296 | syn_misc.at | CGI: EXTERNAL-FORM | 1 | 0 TEXT_ONLY | all passed | 0 |
| 297 | syn_misc.at | adding/removing reserved words | 1 | 0 TEXT_ONLY | 1 | 1 |
| 300 | syn_misc.at | complete specified word list | 1 | 0 TEXT_ONLY | 1 | 1 |
| 301 | syn_misc.at | ANY LENGTH item as formal parameter | 1 | 1 TEXT_ONLY | all passed | 0 |
| 302 | syn_misc.at | ANY LENGTH item as BY VALUE formal parameter | 1 | 0 TEXT_ONLY | all passed | 0 |
| 305 | syn_misc.at | NOT ON EXCEPTION with STATIC CALL convention | 2 | 0 TEXT_ONLY | all passed | 3 |
| 306 | syn_misc.at | NOT ON EXCEPTION phrases before ON EXCEPTION | 1 | 0 TEXT_ONLY | 1 | 1 |
| 307 | syn_misc.at | wrong dialect hints | 1 | 0 TEXT_ONLY | all passed | 0 |
| 308 | syn_misc.at | redundant periods | 1 | 0 TEXT_ONLY | all passed | 0 |
| 309 | syn_misc.at | missing periods | 3 | 0 TEXT_ONLY | all passed | 0 |
| 310 | syn_misc.at | missing periods with COPYs | 3 | 0 TEXT_ONLY | all passed | 1 |
| 312 | syn_misc.at | pseudotext replacement with text in area A | 2 | 0 TEXT_ONLY | all passed | 0 |
| 313 | syn_misc.at | IF-ELSE statement list with invalid syntax | 1 | 0 TEXT_ONLY | all passed | 0 |
| 314 | syn_misc.at | EVALUATE statement with invalid syntax | 1 | 0 TEXT_ONLY | all passed | 0 |
| 315 | syn_misc.at | COBOL-WORDS directive | 1 | 0 TEXT_ONLY | all passed | 0 |
| 316 | syn_misc.at | MF reserved word directives | 1 | 0 TEXT_ONLY | all passed | 0 |
| 317 | syn_misc.at | TURN directive | 1 | 0 TEXT_ONLY | all passed | 0 |
| 318 | syn_misc.at | STRING / UNSTRING with invalid syntax | 1 | 0 TEXT_ONLY | all passed | 0 |
| 319 | syn_misc.at | STRING / UNSTRING POINTER clause | 1 | 0 TEXT_ONLY | all passed | 0 |
| 320 | syn_misc.at | STRING with non-DISPLAY | 1 | 0 TEXT_ONLY | all passed | 0 |
| 321 | syn_misc.at | UNSTRING COUNT clause | 1 | 0 TEXT_ONLY | all passed | 0 |
| 322 | syn_misc.at | use of program-prototype-names | 1 | 0 TEXT_ONLY | all passed | 0 |
| 323 | syn_misc.at | checking prototypes and definitions match | 1 | 0 TEXT_ONLY | all passed | 0 |
| 324 | syn_misc.at | invalid INSPECT/TRANSFORM operands | 1 | 0 TEXT_ONLY | all passed | 0 |
| 325 | syn_misc.at | SIGN clause checks | 1 | 0 TEXT_ONLY | all passed | 0 |
| 326 | syn_misc.at | conflicting entry conventions | 3 | 0 TEXT_ONLY | all passed | 1 |
| 327 | syn_misc.at | conflicting call conventions | 2 | 0 TEXT_ONLY | all passed | 0 |
| 328 | syn_misc.at | dangling LINKAGE items | 1 | 0 TEXT_ONLY | all passed | 0 |
| 329 | syn_misc.at | duplicate PROCEDURE DIVISION/ENTRY USING items | 1 | 0 TEXT_ONLY | all passed | 0 |
| 330 | syn_misc.at | duplicate CALL USING BY REFERENCE items | 1 | 0 TEXT_ONLY | all passed | 0 |
| 331 | syn_misc.at | ADD / SUBTRACT TABLE | 1 | 0 TEXT_ONLY | all passed | 0 |
| 332 | syn_misc.at | USE FOR DEBUGGING invalid ref-mod / subscripts | 1 | 0 TEXT_ONLY | all passed | 0 |
| 333 | syn_misc.at | USE FOR DEBUGGING duplicate targets | 1 | 0 TEXT_ONLY | all passed | 0 |
| 336 | syn_misc.at | USE FOR DEBUGGING syntax-checks (2) | 2 | 0 TEXT_ONLY | all passed | 1 |
| 339 | syn_misc.at | STOP identifier | 1 | 0 TEXT_ONLY | 1 | 1 |
| 340 | syn_misc.at | 01 CONSTANT | 4 | 0 TEXT_ONLY | all passed | 0 |
| 341 | syn_misc.at | 78 VALUE | 3 | 0 TEXT_ONLY | all passed | 0 |
| 342 | syn_misc.at | level 78 NEXT / START OF | 1 | 0 TEXT_ONLY | all passed | 0 |
| 343 | syn_misc.at | SYMBOLIC CONSTANT | 3 | 0 TEXT_ONLY | all passed | 1 |
| 345 | syn_misc.at | Constant Expressions (2) | 1 | 0 TEXT_ONLY | all passed | 0 |
| 349 | syn_misc.at | Missing imperative statements | 2 | 0 TEXT_ONLY | all passed | 1 |
| 350 | syn_misc.at | Fall-Through to WHEN OTHER | 2 | 0 TEXT_ONLY | all passed | 0 |
| 352 | syn_misc.at | ANY LENGTH/NUMERIC with incorrect PIC | 1 | 0 TEXT_ONLY | all passed | 0 |
| 353 | syn_misc.at | VOLATILE clause | 1 | 0 TEXT_ONLY | all passed | 1 |
| 354 | syn_misc.at | SET SOURCEFORMAT syntax checks | 1 | 0 TEXT_ONLY | all passed | 0 |
| 355 | syn_misc.at | WHEN-COMPILED register in dialect | 1 | 0 TEXT_ONLY | all passed | 0 |
| 356 | syn_misc.at | LIN / COL register | 3 | 0 TEXT_ONLY | all passed | 0 |
| 358 | syn_misc.at | @OPTIONS parsing | 1 | 0 TEXT_ONLY | all passed | 0 |
| 359 | syn_misc.at | PROCESS / CBL parsing | 1 | 0 TEXT_ONLY | all passed | 0 |
| 361 | syn_misc.at | system routines with wrong number of parameters | 1 | 0 TEXT_ONLY | all passed | 0 |
| 362 | syn_misc.at | invalid use of condition-name | 1 | 0 TEXT_ONLY | all passed | 0 |
| 363 | syn_misc.at | XML GENERATE syntax checks | 1 | 0 TEXT_ONLY | all passed | 0 |
| 364 | syn_misc.at | BASED clause, ALLOCATE / FREE statements | 2 | 0 TEXT_ONLY | all passed | 1 |
| 365 | syn_misc.at | CONTINUE statement | 2 | 0 TEXT_ONLY | all passed | 1 |
| 366 | syn_misc.at | conflict markers | 2 | 0 TEXT_ONLY | all passed | 1 |
| 367 | syn_misc.at | SORT syntax | 1 | 0 TEXT_ONLY | all passed | 0 |
| 368 | syn_misc.at | OSVS I/O extensions | 1 | 0 TEXT_ONLY | all passed | 0 |
| 370 | syn_misc.at | SEARCH ALL checks | 1 | 0 TEXT_ONLY | all passed | 0 |
| 371 | syn_misc.at | Invalid parentheses around condition | 1 | 0 TEXT_ONLY | all passed | 0 |
| 390 | syn_misc.at | CONTROL: default section | 2 | 0 TEXT_ONLY | all passed | 0 |
| 392 | syn_misc.at | CONTROL DIVISION & AREACHECK | 1 | 0 TEXT_ONLY | all passed | 0 |
| 393 | syn_misc.at | PICTURE L | 5 | 0 TEXT_ONLY | all passed | 0 |
| 394 | syn_misc.at | AREACHECK / NOAREACHECK directives | 2 | 0 TEXT_ONLY | all passed | 1 |
| 395 | syn_misc.at | AREACHECK / NOAREACHECK directives (2) | 3 | 0 TEXT_ONLY | all passed | 2 |
| 396 | syn_misc.at | optional dots | 2 | 0 TEXT_ONLY | all passed | 0 |
| 397 | syn_misc.at | optional dots before PROCEDURE DIVISION | 2 | 0 TEXT_ONLY | all passed | 0 |
| 398 | syn_misc.at | AREACHECK | 2 | 0 TEXT_ONLY | all passed | 1 |
| 399 | syn_misc.at | autodetect format | 4 | 3 TEXT_ONLY | all passed | 0 |
| 401 | syn_move.at | MOVE SPACE TO numeric or numeric-edited item | 1 | 0 TEXT_ONLY | all passed | 0 |
| 402 | syn_move.at | MOVE ZERO TO alphabetic item | 1 | 0 TEXT_ONLY | all passed | 0 |
| 403 | syn_move.at | MOVE alphabetic TO x | 1 | 0 TEXT_ONLY | all passed | 0 |
| 405 | syn_move.at | MOVE alphanumeric-edited TO x | 1 | 0 TEXT_ONLY | all passed | 0 |
| 406 | syn_move.at | MOVE numeric (integer) TO x | 1 | 0 TEXT_ONLY | all passed | 0 |
| 407 | syn_move.at | MOVE numeric (non-integer) TO x | 1 | 0 TEXT_ONLY | all passed | 0 |
| 408 | syn_move.at | MOVE numeric-edited TO x | 1 | 0 TEXT_ONLY | all passed | 0 |
| 409 | syn_move.at | MOVE national TO x | 1 | 0 TEXT_ONLY | all passed | 0 |
| 410 | syn_move.at | MOVE national-edited TO x | 1 | 0 TEXT_ONLY | all passed | 0 |
| 411 | syn_move.at | CORRESPONDING - Operands must be groups | 1 | 0 TEXT_ONLY | all passed | 0 |
| 412 | syn_move.at | CORRESPONDING - Target has no matching items | 1 | 0 TEXT_ONLY | all passed | 0 |
| 413 | syn_move.at | MOVE to erroneous field | 1 | 0 TEXT_ONLY | all passed | 0 |
| 414 | syn_move.at | Overlapping MOVE | 2 | 1 TEXT_ONLY | all passed | 1 |
| 415 | syn_move.at | invalid source for MOVE | 1 | 0 TEXT_ONLY | all passed | 0 |
| 416 | syn_move.at | invalid target for MOVE | 1 | 0 TEXT_ONLY | all passed | 0 |
| 417 | syn_move.at | SET error | 1 | 0 TEXT_ONLY | all passed | 0 |
| 418 | syn_move.at | MOVE FIGURATIVE to NUMERIC | 4 | 0 TEXT_ONLY | all passed | 1 |
| 419 | syn_multiply.at | Category check of Format 1 | 1 | 0 TEXT_ONLY | all passed | 0 |
| 420 | syn_multiply.at | Category check of Format 2 | 1 | 0 TEXT_ONLY | all passed | 0 |
| 421 | syn_multiply.at | Category check of literals | 1 | 0 TEXT_ONLY | all passed | 0 |
| 422 | syn_screen.at | Flexible ACCEPT/DISPLAY syntax | 1 | 0 TEXT_ONLY | all passed | 0 |
| 423 | syn_screen.at | Duplicate ACCEPT/DISPLAY clauses | 1 | 0 TEXT_ONLY | all passed | 0 |
| 424 | syn_screen.at | AT clause | 1 | 0 TEXT_ONLY | all passed | 0 |
| 425 | syn_screen.at | ACCEPT/DISPLAY extensions detection | 1 | 0 TEXT_ONLY | all passed | 0 |
| 426 | syn_screen.at | FROM clause | 1 | 0 TEXT_ONLY | all passed | 0 |
| 427 | syn_screen.at | Incorrect USAGE clause | 1 | 0 TEXT_ONLY | all passed | 0 |
| 428 | syn_screen.at | SCREEN SECTION clause numbers | 1 | 0 TEXT_ONLY | all passed | 0 |
| 429 | syn_screen.at | Screen clauses | 1 | 0 TEXT_ONLY | all passed | 0 |
| 431 | syn_screen.at | Referencing 88-level | 1 | 0 TEXT_ONLY | all passed | 0 |
| 432 | syn_screen.at | Conflicting screen clauses | 2 | 0 TEXT_ONLY | all passed | 1 |
| 433 | syn_screen.at | Redundant screen clauses | 1 | 0 TEXT_ONLY | all passed | 0 |
| 434 | syn_screen.at | Screen item OCCURS w-/wo relative LINE/COL | 3 | 0 TEXT_ONLY | all passed | 2 |
| 435 | syn_screen.at | VALUE clause missing | 1 | 0 TEXT_ONLY | 1 | 1 |
| 436 | syn_screen.at | FULL on numeric item | 1 | 0 TEXT_ONLY | all passed | 0 |
| 437 | syn_screen.at | Compiler-specific SCREEN SECTION clause rules | 6 | 0 TEXT_ONLY | all passed | 5 |
| 438 | syn_screen.at | MS-COBOL position-spec | 1 | 0 TEXT_ONLY | all passed | 0 |
| 439 | syn_screen.at | Screen with invalid FROM clause | 2 | 0 TEXT_ONLY | all passed | 1 |
| 440 | syn_screen.at | DISPLAY WITH CONVERSION | 1 | 0 TEXT_ONLY | all passed | 0 |
| 441 | syn_set.at | SET ADDRESS OF item | 1 | 0 TEXT_ONLY | all passed | 0 |
| 442 | syn_set.at | SET item TO 88-level | 1 | 0 TEXT_ONLY | all passed | 0 |
| 443 | syn_functions.at | ANY LENGTH / NUMERIC as function RETURNING item | 2 | 0 TEXT_ONLY | all passed | 1 |
| 446 | syn_functions.at | Redundant REPOSITORY entries | 1 | 0 TEXT_ONLY | all passed | 0 |
| 447 | syn_functions.at | Missing prototype/definition | 1 | 0 TEXT_ONLY | all passed | 0 |
| 448 | syn_functions.at | Empty function | 1 | 0 TEXT_ONLY | all passed | 0 |
| 449 | syn_functions.at | Function definition inside program | 1 | 0 TEXT_ONLY | all passed | 0 |
| 450 | syn_functions.at | Intrinsic functions: dialect | 1 | 1 TEXT_ONLY | all passed | 0 |
| 452 | syn_functions.at | Intrinsic functions: number of arguments | 1 | 0 TEXT_ONLY | all passed | 0 |
| 453 | syn_functions.at | Intrinsic functions: reference modification | 1 | 0 TEXT_ONLY | all passed | 0 |
| 454 | syn_functions.at | Intrinsic functions: argument type | 1 | 0 TEXT_ONLY | all passed | 0 |
| 455 | syn_functions.at | invalid formatted date/time args | 1 | 0 TEXT_ONLY | all passed | 0 |
| 456 | syn_functions.at | invalid formats w/ DECIMAL-POINT IS COMMA | 1 | 0 TEXT_ONLY | all passed | 0 |
| 457 | syn_functions.at | Specified offset and SYSTEM-OFFSET | 1 | 0 TEXT_ONLY | all passed | 0 |
| 458 | syn_functions.at | FUNCTION LENGTH / BYTE-LENGTH | 1 | 0 TEXT_ONLY | all passed | 0 |
| 459 | syn_literals.at | continuation Indicator - too many lines | 2 | 0 TEXT_ONLY | all passed | 0 |
| 460 | syn_literals.at | literal too long | 5 | 0 TEXT_ONLY | all passed | 0 |
| 462 | syn_literals.at | floating-point literals | 2 | 0 TEXT_ONLY | all passed | 1 |
| 463 | syn_literals.at | X literals | 2 | 0 TEXT_ONLY | all passed | 0 |
| 464 | syn_literals.at | national literals | 2 | 0 TEXT_ONLY | all passed | 1 |
| 465 | syn_literals.at | NX literals | 2 | 0 TEXT_ONLY | all passed | 0 |
| 466 | syn_literals.at | binary literals | 2 | 0 TEXT_ONLY | all passed | 0 |
| 467 | syn_literals.at | binary-hexadecimal literals | 2 | 0 TEXT_ONLY | all passed | 0 |
| 468 | syn_literals.at | HP COBOL octal literals | 2 | 0 TEXT_ONLY | all passed | 1 |
| 469 | syn_literals.at | ACUCOBOL literals | 2 | 0 TEXT_ONLY | all passed | 0 |
| 471 | syn_literals.at | zero-length literals | 2 | 0 TEXT_ONLY | all passed | 1 |
| 472 | syn_literals.at | long literal in error message | 1 | 0 TEXT_ONLY | all passed | 0 |
| 473 | syn_literals.at | literal missing terminating character | 1 | 0 TEXT_ONLY | all passed | 0 |
| 474 | syn_literals.at | GCOS literals with EBCDIC symbols (syntax) | 3 | 0 TEXT_ONLY | 1 | 1 |
| 475 | listings.at | Minimal lines per listing pages | 1 | 0 TEXT_ONLY | all passed | 0 |
| 521 | run_fundamental.at | MOVE integer literal to alphanumeric | 3 | 0 TEXT_ONLY | all passed | 4 |
| 525 | run_fundamental.at | Overlapping MOVE (GnuCOBOL) | 1 | 0 TEXT_ONLY | 4 | 4 |
| 576 | run_fundamental.at | Numeric operations (8) | 1 | 0 TEXT_ONLY | 1 | 1 |
| 578 | run_fundamental.at | ADD CORRESPONDING no match | 1 | 0 TEXT_ONLY | all passed | 1 |
| 605 | run_fundamental.at | Abbreviated Expressions | 2 | 0 TEXT_ONLY | 1 | 1 |
| 608 | run_fundamental.at | Alphanumeric VALUE longer than PIC | 1 | 0 TEXT_ONLY | all passed | 1 |
| 610 | run_fundamental.at | condition IS ZERO AND | 1 | 0 TEXT_ONLY | all passed | 1 |
| 612 | run_fundamental.at | abbreviated conditions with multiple words operators | 1 | 0 TEXT_ONLY | all passed | 0 |
| 683 | run_misc.at | MOVE to itself | 1 | 0 TEXT_ONLY | all passed | 1 |
| 729 | run_misc.at | PERFORM VARYING BY phrase omitted | 1 | 0 TEXT_ONLY | 2 | 2 |
| 754 | run_misc.at | Alphanum comparison with default COLLATING SEQUENCE | 1 | 0 TEXT_ONLY | all passed | 0 |
| 802 | run_misc.at | Figurative constants to numeric field | 1 | 0 TEXT_ONLY | 1 | 1 |
| 803 | run_misc.at | MF FIGURATIVE to NUMERIC | 1 | 0 TEXT_ONLY | all passed | 0 |
| 811 | run_misc.at | REDEFINES values on FILLER and INITIALIZE | 1 | 0 TEXT_ONLY | all passed | 1 |
| 812 | run_misc.at | PICTURE with constant-name | 1 | 0 TEXT_ONLY | all passed | 0 |
| 813 | run_misc.at | Quote marks in comment paragraphs | 2 | 0 TEXT_ONLY | 2 | 2 |
| 814 | run_misc.at | Numeric MOVE with/without -fbinary-truncate | 1 | 0 TEXT_ONLY | 2 | 2 |
| 816 | run_misc.at | PROGRAM-ID / CALL literal/variable with spaces | 1 | 0 TEXT_ONLY | 1 | 1 |
| 819 | run_misc.at | C-API (param based) | 1 | 0 TEXT_ONLY | 1 | 1 |
| 820 | run_misc.at | C-API (field based) | 1 | 0 TEXT_ONLY | 1 | 1 |
| 822 | run_misc.at | OCCURS INDEXED ASCENDING | 1 | 0 TEXT_ONLY | all passed | 0 |
| 826 | run_misc.at | OSVS Arithmetic (1) | 1 | 0 TEXT_ONLY | all passed | 0 |
| 827 | run_misc.at | OSVS Arithmetic Test (2) | 1 | 0 TEXT_ONLY | 1 | 1 |
| 833 | run_misc.at | DISPLAY UPON | 1 | 0 TEXT_ONLY | 1 | 1 |
| 841 | run_misc.at | ENTRY FOR GO TO / GO TO ENTRY | 2 | 0 TEXT_ONLY | all passed | 0 |
| 860 | run_file.at | ASSIGN EXTERNAL parsing | 1 | 0 TEXT_ONLY | all passed | 0 |
| 966 | run_reportwriter.at | Sample REPORT with RIGHT/CENTER | 2 | 0 TEXT_ONLY | all passed | 0 |
| 970 | run_reportwriter.at | Sample Inventory Report | 1 | 0 TEXT_ONLY | 1 | 1 |
| 973 | run_reportwriter.at | Report CODE and LIMIT COLUMNS | 2 | 0 TEXT_ONLY | all passed | 0 |
| 990 | run_functions.at | FUNCTION BYTE-LENGTH | 1 | 0 TEXT_ONLY | 1 | 1 |
| 1103 | run_functions.at | Formatted funcs w/ invalid variable format | 1 | 0 TEXT_ONLY | 1 | 1 |
| 1134 | run_extensions.at | MOVE of non-integer to alphanumeric | 1 | 0 TEXT_ONLY | all passed | 1 |
| 1138 | run_extensions.at | CALL BY VALUE alphanumeric item | 1 | 0 TEXT_ONLY | all passed | 1 |
| 1145 | run_extensions.at | TALLY register | 1 | 0 TEXT_ONLY | 2 | 2 |
| 1147 | run_extensions.at | PROCEDURE DIVISION USING BY ... | 1 | 1 TEXT_ONLY | all passed | 1 |
| 1161 | run_extensions.at | Obsolete 2002 keywords with COBOL2014 | 1 | 0 TEXT_ONLY | all passed | 0 |
| 1162 | run_extensions.at | System routine with wrong number of parameters | 2 | 0 TEXT_ONLY | all passed | 0 |
| 1189 | run_extensions.at | CALL own PROGRAM-ID and RECURSIVE attribute | 3 | 0 TEXT_ONLY | all passed | 3 |
| 1195 | run_extensions.at | Invalid source format | 2 | 0 TEXT_ONLY | all passed | 0 |

_Generated from committed raw evidence; raw samples are preserved._
