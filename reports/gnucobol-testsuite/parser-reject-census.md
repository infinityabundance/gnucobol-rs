# GnuCOBOL testsuite — parser/checker rejection census

**683 first-failure groups** whose primary classification is a candidate check/parse reject, decomposed by PHASE (checker/data-layout/grammar/name-resolution/semantic-check) and by diagnostic. A construct rejected at run (the launcher ran) is attributed the same way as at syntax-only (first-failure consistency). Counting unit: **first_failure_group** (one row per unique group; step-level AT_CHECK identities are not counted here and are never labelled "tests").

## Phases

- checker: 406
- data-layout: 98
- grammar: 115
- name-resolution: 31
- semantic-check: 33

## Top diagnostics

- 234: ``
- 63: `cobc-rs: unsupported: no PROCEDURE DIVISION`
- 22: `cobrun: unsupported: unsupported level number REPLACE`
- 14: `cobrun: unsupported: not a numeric literal: X`
- 9: `cobrun: unsupported: no PROCEDURE DIVISION`
- 8: `cobc-rs: unsupported: unsupported level number SCREEN`
- 8: `cobc-rs: unsupported: verb USE`
- 8: `cobrun: unsupported: DELETE: `FILE` is not a declared file`
- 7: `cobrun: unsupported: not a numeric literal: ALL`
- 7: `cobrun: unsupported: not a numeric literal: FUNCTION`
- 6: `cobc-rs: unsupported: PIC BX: UnsupportedSymbol('X')`
- 6: `"cobrun: unsupported: CALL ""dump"": not a contained program (external CALL is a boundary)"`
- 6: `cobrun: unsupported: OCCURS count MAX-SUB is not an integer`
- 6: `cobrun: unsupported: expected program name after PROGRAM-ID`
- 6: `cobrun: unsupported: not a numeric literal: FENCE`
- 5: `cobrun: unsupported: ACCEPT FROM DATE/TIME requires a pinned COB_CURRENT_DATE (the live clock is a non-claim)`
- 5: `cobrun: unsupported: unsupported level number LOCAL-STORAGE`
- 5: `cobrun: unsupported: verb CHAINING`
- 4: `cobc-rs: unsupported: no PROGRAM-ID`
- 4: `cobrun: undefined data name: FUNCTION`
- 4: `cobrun: unsupported: OPEN: `FILE-OPT` is not a declared file`
- 4: `"cobrun: unsupported: OPEN: `TRANSACTION-DATA,` is not a declared file"`
- 3: `cannot read prog.cob: No such file or directory (os error 2)`
- 3: `cobc-rs: unsupported: verb IDENTIFICATION`
- 3: `cobrun: unsupported: OCCURS max UNBOUNDED is not an integer`
- 3: `cobrun: unsupported: OPEN: `OUTPUT` is not a declared file`
- 3: `cobrun: unsupported: SORT: `TBL` is not a declared file`
- 3: `cobrun: unsupported: condition: unrecognized relational operator (expected = > < >= <= <> GREATER LESS EQUAL)`
- 3: `"cobrun: unsupported: not a numeric literal: FOO,"`
- 3: `cobrun: unsupported: not a numeric literal: NULL`
- 3: `cobrun: unsupported: not a numeric literal: Z`
- 2: `cobc-rs: unsupported: PIC N: UnsupportedSymbol('N')`
- 2: `cobc-rs: unsupported: PIC VPP99: ScalingPDeferred`
- 2: `cobc-rs: unsupported: unrecognized USAGE BIT`
- 2: `cobc-rs: unsupported: unsupported level number $SET`
- 2: `cobc-rs: unsupported: verb END-OF-FILE-SWITCH`
- 2: `cobrun: undefined data name: ENVIRONMENT`
- 2: `cobrun: undefined data name: NULL`
- 2: `cobrun: unsupported: ACCEPT FROM terminal/console: interactive input is a runtime non-claim (no deterministic oracle); the wired sources are DATE/DAY/TIME/DAY-OF-WEEK/ENVIRONMENT`
- 2: `"cobrun: unsupported: CALL ""C$GETPID"": not a contained program (external CALL is a boundary)"`

## Per-test ledger (counting_unit = first_failure_group)

| id | title | phase | diagnostic | classification |
|---|---|---|---|---|
| 0002 | used_binaries.at:51: 2. compiler warnings | checker |  | CANDIDATE_CHECK_REJECT |
| 0005 | used_binaries.at:300: 5. compiler outputs (path specified) | checker | cannot read sub/prog.c: stream did not contain valid UTF-8 | CANDIDATE_CHECK_REJECT |
| 0007 | used_binaries.at:364: 7. source file not found | checker | cannot read prog.cob: No such file or directory (os error 2) | CANDIDATE_CHECK_REJECT |
| 0008 | used_binaries.at:374: 8. temporary path invalid | checker |  | CANDIDATE_CHECK_REJECT |
| 0019 | used_binaries.at:796: 19. run job with unsuccessful compilation | checker |  | CANDIDATE_CHECK_REJECT |
| 0022 | used_binaries.at:850: 22. run job with optional arguments | checker |  | CANDIDATE_CHECK_REJECT |
| 0026 | configuration.at:22: 26. cobc with standard configuration file | checker |  | CANDIDATE_CHECK_REJECT |
| 0027 | configuration.at:43: 27. cobc dialect features for all -std | checker |  | CANDIDATE_CHECK_REJECT |
| 0028 | configuration.at:105: 28. cobc with configuration file via -std | checker |  | CANDIDATE_CHECK_REJECT |
| 0029 | configuration.at:126: 29. cobc with standard configuration file via -conf | checker | "cobc-rs: warning: cannot read dialect config ""cobol2014.conf""; using -std/default" | CANDIDATE_CHECK_REJECT |
| 0031 | configuration.at:176: 31. cobc configuration: recursive include | checker |  | CANDIDATE_CHECK_REJECT |
| 0032 | configuration.at:214: 32. cobc with -std and -conf | checker |  | CANDIDATE_CHECK_REJECT |
| 0036 | configuration.at:340: 36. cobc configuration: conf missing | checker | cannot read prog.cob: No such file or directory (os error 2) | CANDIDATE_CHECK_REJECT |
| 0037 | configuration.at:368: 37. cobc configuration: conf optional | checker |  | CANDIDATE_CHECK_REJECT |
| 0038 | configuration.at:404: 38. cobc configuration: incomplete | checker | cannot read prog.cob: No such file or directory (os error 2) | CANDIDATE_CHECK_REJECT |
| 0048 | configuration.at:827: 48. cobc configuration: source format | checker | cobc-rs: -fformat=unknown: only fixed/free/auto are supported by the candidate (fail closed) | CANDIDATE_CHECK_REJECT |
| 0055 | syn_copy.at:342: 55. COPY: recursive | checker | cobc-rs: COPY expansion failed (fail closed): copybook 'COPY1' not found | CANDIDATE_CHECK_REJECT |
| 0073 | syn_definition.at:25: 73. Invalid source name | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0074 | syn_definition.at:37: 74. Invalid PROGRAM-ID | checker |  | CANDIDATE_CHECK_REJECT |
| 0075 | syn_definition.at:89: 75. Invalid PROGRAM-ID type clause (1) | checker |  | CANDIDATE_CHECK_REJECT |
| 0076 | syn_definition.at:106: 76. invalid PROGRAM-ID type clause (2) | checker |  | CANDIDATE_CHECK_REJECT |
| 0077 | syn_definition.at:123: 77. INITIAL / RECURSIVE before COMMON | grammar | cobc-rs: unsupported: verb IDENTIFICATION | CANDIDATE_CHECK_REJECT |
| 0078 | syn_definition.at:155: 78. Undefined data name | checker |  | CANDIDATE_CHECK_REJECT |
| 0079 | syn_definition.at:173: 79. Undefined group name | checker |  | CANDIDATE_CHECK_REJECT |
| 0080 | syn_definition.at:194: 80. Undefined data name in group | checker |  | CANDIDATE_CHECK_REJECT |
| 0081 | syn_definition.at:217: 81. Reference not a group name | checker |  | CANDIDATE_CHECK_REJECT |
| 0082 | syn_definition.at:239: 82. Incomplete 01 definition | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0083 | syn_definition.at:257: 83. error handling in conditions | grammar | cobc-rs: unsupported: verb NOT-DEFINED | CANDIDATE_CHECK_REJECT |
| 0084 | syn_definition.at:333: 84. Same paragraphs in different sections | checker |  | CANDIDATE_CHECK_REJECT |
| 0085 | syn_definition.at:376: 85. GO TO sections and foreign paragraphs | checker |  | CANDIDATE_CHECK_REJECT |
| 0086 | syn_definition.at:413: 86. Redefinition of 01 items | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0087 | syn_definition.at:442: 87. Redefinition of 01 and 02 items | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0088 | syn_definition.at:462: 88. Redefinition of 02 items | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0089 | syn_definition.at:495: 89. Redefinition of 77 items | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0090 | syn_definition.at:515: 90. Redefinition of 01 and 77 items | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0091 | syn_definition.at:535: 91. Redefinition of 88 items | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0093 | syn_definition.at:636: 93. Redefinition of program-name within program | checker |  | CANDIDATE_CHECK_REJECT |
| 0094 | syn_definition.at:666: 94. Redefinition of function-prototype name | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0095 | syn_definition.at:691: 95. PROCEDURE DIVISION RETURNING OMITTED: main | checker |  | CANDIDATE_CHECK_REJECT |
| 0096 | syn_definition.at:711: 96. PROCEDURE DIVISION RETURNING OMITTED: FUNCTION | checker | cobc-rs: unsupported: no PROGRAM-ID | CANDIDATE_CHECK_REJECT |
| 0097 | syn_definition.at:730: 97. PROCEDURE DIVISION RETURNING item | checker | cobc-rs: unsupported: no PROGRAM-ID | CANDIDATE_CHECK_REJECT |
| 0098 | syn_definition.at:825: 98. Data item with same name as program-name | checker | cobc-rs: unsupported: PROG: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0099 | syn_definition.at:852: 99. Ambiguous reference to 02 items | checker |  | CANDIDATE_CHECK_REJECT |
| 0100 | syn_definition.at:878: 100. Ambiguous reference to 02 and 03 items | checker |  | CANDIDATE_CHECK_REJECT |
| 0101 | syn_definition.at:903: 101. Ambiguous reference with qualification | checker |  | CANDIDATE_CHECK_REJECT |
| 0103 | syn_definition.at:955: 103. SYNCHRONIZED clause | checker |  | CANDIDATE_CHECK_REJECT |
| 0104 | syn_definition.at:998: 104. Undefined procedure name | checker |  | CANDIDATE_CHECK_REJECT |
| 0105 | syn_definition.at:1018: 105. Redefinition of section names | checker |  | CANDIDATE_CHECK_REJECT |
| 0106 | syn_definition.at:1043: 106. Redefinition of section and paragraph names | checker |  | CANDIDATE_CHECK_REJECT |
| 0109 | syn_definition.at:1132: 109. Ambiguous reference to paragraph name | checker |  | CANDIDATE_CHECK_REJECT |
| 0111 | syn_definition.at:1190: 111. CALL BY VALUE alphanumeric item (extension) | checker |  | CANDIDATE_CHECK_REJECT |
| 0112 | syn_definition.at:1212: 112. CALL BY VALUE national item (extension) | data-layout | cobc-rs: unsupported: PIC N(4): UnsupportedSymbol('N') | CANDIDATE_CHECK_REJECT |
| 0114 | syn_definition.at:1269: 114. Duplicate identification division header | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0118 | syn_definition.at:1430: 118. Function without END FUNCTION | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0119 | syn_definition.at:1444: 119. Nested programs without END PROGRAM | checker | cobc-rs: unsupported: PROG-3: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0120 | syn_definition.at:1466: 120. Nested programs not in procedure division | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0121 | syn_definition.at:1485: 121. Screen section starts with 78-level | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0122 | syn_definition.at:1501: 122. Invalid PICTURE strings | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0123 | syn_definition.at:1761: 123. PICTURE string with control character | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0124 | syn_definition.at:1779: 124. PICTURE strings invalid with BLANK WHEN ZERO | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0125 | syn_definition.at:1802: 125. PICTURE strings invalid with USAGE | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0126 | syn_definition.at:1822: 126. Edited monetary PICTURE strings | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0127 | syn_definition.at:1849: 127. ALPHABET definition | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0128 | syn_definition.at:1872: 128. PROGRAM COLLATING SEQUENCE | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0129 | syn_definition.at:2055: 129. RENAMES item | checker |  | CANDIDATE_CHECK_REJECT |
| 0130 | "syn_definition.at:2129: 130. RENAMES of 01-, 66- and 77-level items" | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0131 | syn_definition.at:2155: 131. SAME AS clause | checker |  | CANDIDATE_CHECK_REJECT |
| 0132 | syn_definition.at:2219: 132. TYPEDEF clause | data-layout | cobc-rs: unsupported: unrecognized USAGE SOME-VERY-LONG-TYPEDEF-NAME | CANDIDATE_CHECK_REJECT |
| 0134 | syn_definition.at:2629: 134. APPLY COMMIT clause | grammar | cobc-rs: unsupported: verb INITIALISATION | CANDIDATE_CHECK_REJECT |
| 0135 | syn_definition.at:2734: 135. GLOBAL record-names | grammar | cobc-rs: unsupported: verb KEY | CANDIDATE_CHECK_REJECT |
| 0136 | syn_definition.at:2853: 136. Invalid USE BEFORE | grammar | cobc-rs: unsupported: verb USE | CANDIDATE_CHECK_REJECT |
| 0137 | syn_subscripts.at:23: 137. Non-numeric subscript | checker |  | CANDIDATE_CHECK_REJECT |
| 0138 | syn_subscripts.at:50: 138. Subscript range check | checker |  | CANDIDATE_CHECK_REJECT |
| 0139 | syn_subscripts.at:98: 139. Subscript bounds with OCCURS DEPENDING ON | checker |  | CANDIDATE_CHECK_REJECT |
| 0140 | syn_subscripts.at:125: 140. Subscripted item requires OCCURS clause | checker |  | CANDIDATE_CHECK_REJECT |
| 0141 | syn_subscripts.at:151: 141. Number of subscripts | checker |  | CANDIDATE_CHECK_REJECT |
| 0142 | syn_subscripts.at:195: 142. SET SSRANGE syntax | checker |  | CANDIDATE_CHECK_REJECT |
| 0143 | syn_occurs.at:29: 143. OCCURS with level 01 and 77 | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0144 | syn_occurs.at:84: 144. OCCURS with level 66 | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0145 | syn_occurs.at:103: 145. OCCURS with level 78 | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0146 | syn_occurs.at:121: 146. OCCURS with level 88 | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0147 | syn_occurs.at:143: 147. OCCURS with variable-occurrence data item | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0148 | syn_occurs.at:248: 148. OCCURS UNBOUNDED | data-layout | cobc-rs: unsupported: OCCURS count UNBOUNDED is not an integer | CANDIDATE_CHECK_REJECT |
| 0149 | syn_occurs.at:359: 149. OCCURS data-items for INDEXED and KEY | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0150 | syn_occurs.at:399: 150. Nested OCCURS clause | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0151 | syn_occurs.at:436: 151. OCCURS DEPENDING with wrong size | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0152 | syn_occurs.at:475: 152. OCCURS DEPENDING followed by another field | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0153 | syn_occurs.at:517: 153. OCCURS with unmatched DEPENDING / TO phrases | checker |  | CANDIDATE_CHECK_REJECT |
| 0154 | syn_occurs.at:560: 154. OCCURS INDEXED before KEY | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0155 | syn_occurs.at:598: 155. OCCURS size check | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0157 | syn_redefines.at:53: 157. REDEFINES: level 02 by 01 | checker |  | CANDIDATE_CHECK_REJECT |
| 0158 | syn_redefines.at:74: 158. REDEFINES: level 03 by 02 | checker |  | CANDIDATE_CHECK_REJECT |
| 0159 | syn_redefines.at:96: 159. REDEFINES: level 66 | checker | cobc-rs: unsupported: 66 level without RENAMES | CANDIDATE_CHECK_REJECT |
| 0160 | syn_redefines.at:118: 160. REDEFINES: level 88 | checker |  | CANDIDATE_CHECK_REJECT |
| 0161 | syn_redefines.at:146: 161. REDEFINES: lower level number | checker |  | CANDIDATE_CHECK_REJECT |
| 0162 | syn_redefines.at:172: 162. REDEFINES: with OCCURS | checker |  | CANDIDATE_CHECK_REJECT |
| 0163 | syn_redefines.at:196: 163. REDEFINES: with subscript | checker |  | CANDIDATE_CHECK_REJECT |
| 0164 | syn_redefines.at:219: 164. REDEFINES: with variable occurrence | checker |  | CANDIDATE_CHECK_REJECT |
| 0165 | syn_redefines.at:254: 165. REDEFINES: with qualification | checker |  | CANDIDATE_CHECK_REJECT |
| 0166 | syn_redefines.at:280: 166. REDEFINES: multiple redefinition | checker |  | CANDIDATE_CHECK_REJECT |
| 0167 | syn_redefines.at:308: 167. REDEFINES: size exceeds | checker |  | CANDIDATE_CHECK_REJECT |
| 0168 | syn_redefines.at:359: 168. REDEFINES: with VALUE | checker |  | CANDIDATE_CHECK_REJECT |
| 0169 | syn_redefines.at:392: 169. REDEFINES: with intervention | checker |  | CANDIDATE_CHECK_REJECT |
| 0171 | syn_redefines.at:467: 171. REDEFINES: for ANY LENGTH item | checker |  | CANDIDATE_CHECK_REJECT |
| 0172 | syn_redefines.at:500: 172. REDEFINES: non-referenced ambiguous item | checker |  | CANDIDATE_CHECK_REJECT |
| 0173 | syn_value.at:28: 173. bad VALUES / VALUES ARE in format-1 | grammar | cobc-rs: unsupported: not a numeric literal: ARE | CANDIDATE_CHECK_REJECT |
| 0174 | syn_value.at:76: 174. OCCURS too many VALUEs | checker |  | CANDIDATE_CHECK_REJECT |
| 0175 | syn_value.at:162: 175. Numeric item (integer) | checker |  | CANDIDATE_CHECK_REJECT |
| 0176 | syn_value.at:189: 176. Numeric item (non-integer) | checker |  | CANDIDATE_CHECK_REJECT |
| 0177 | syn_value.at:213: 177. Numeric item with picture P | checker |  | CANDIDATE_CHECK_REJECT |
| 0178 | syn_value.at:245: 178. Signed numeric literal | checker |  | CANDIDATE_CHECK_REJECT |
| 0179 | syn_value.at:271: 179. Alphabetic item | checker |  | CANDIDATE_CHECK_REJECT |
| 0180 | syn_value.at:299: 180. Alphanumeric item | checker |  | CANDIDATE_CHECK_REJECT |
| 0181 | syn_value.at:325: 181. Alphanumeric group item | checker |  | CANDIDATE_CHECK_REJECT |
| 0182 | syn_value.at:352: 182. National item | data-layout | cobc-rs: unsupported: PIC NNN: UnsupportedSymbol('N') | CANDIDATE_CHECK_REJECT |
| 0183 | syn_value.at:389: 183. Numeric-edited item | checker |  | CANDIDATE_CHECK_REJECT |
| 0184 | syn_value.at:425: 184. Alphanumeric-edited item | data-layout | cobc-rs: unsupported: PIC BXX: UnsupportedSymbol('X') | CANDIDATE_CHECK_REJECT |
| 0186 | syn_file.at:23: 186. Missing SELECT | checker |  | CANDIDATE_CHECK_REJECT |
| 0187 | syn_file.at:50: 187. Duplicated SELECT | checker |  | CANDIDATE_CHECK_REJECT |
| 0188 | syn_file.at:82: 188. Missing FD | checker |  | CANDIDATE_CHECK_REJECT |
| 0189 | syn_file.at:108: 189. Duplicated FD | checker |  | CANDIDATE_CHECK_REJECT |
| 0194 | syn_file.at:431: 194. SELECT without ASSIGN | checker |  | CANDIDATE_CHECK_REJECT |
| 0195 | syn_file.at:459: 195. START on SEQUENTIAL file | checker |  | CANDIDATE_CHECK_REJECT |
| 0196 | syn_file.at:496: 196. OPEN SEQUENTIAL file REVERSED | checker |  | CANDIDATE_CHECK_REJECT |
| 0197 | syn_file.at:544: 197. OPEN SEQUENTIAL file NO REWIND | checker |  | CANDIDATE_CHECK_REJECT |
| 0199 | syn_file.at:633: 199. INDEXED file invalid key items | checker |  | CANDIDATE_CHECK_REJECT |
| 0200 | syn_file.at:697: 200. variable record length | checker |  | CANDIDATE_CHECK_REJECT |
| 0201 | syn_file.at:808: 201. variable record length DEPENDING item | checker |  | CANDIDATE_CHECK_REJECT |
| 0202 | syn_file.at:910: 202. DECLARATIVES invalid procedure reference (1) | grammar | cobc-rs: unsupported: verb USE | CANDIDATE_CHECK_REJECT |
| 0203 | syn_file.at:999: 203. DECLARATIVES invalid procedure reference (2) | grammar | cobc-rs: unsupported: verb USE | CANDIDATE_CHECK_REJECT |
| 0205 | syn_file.at:1068: 205. RECORDING MODE | checker |  | CANDIDATE_CHECK_REJECT |
| 0206 | syn_file.at:1097: 206. CODE-SET clause | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0207 | syn_file.at:1147: 207. CODE-SET FOR clause | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0208 | syn_file.at:1180: 208. WRITE / REWRITE FROM clause and FILE | checker |  | CANDIDATE_CHECK_REJECT |
| 0209 | syn_file.at:1241: 209. Clauses following invalid ACCESS clause | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0210 | syn_file.at:1265: 210. RELATIVE KEY type validation | checker |  | CANDIDATE_CHECK_REJECT |
| 0211 | syn_file.at:1325: 211. Mismatched KEY clause | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0214 | syn_file.at:1557: 214. VSAM status | checker |  | CANDIDATE_CHECK_REJECT |
| 0215 | syn_file.at:1603: 215. INDEXED file PASSWORD clause | checker |  | CANDIDATE_CHECK_REJECT |
| 0216 | syn_file.at:1654: 216. RECORD clause equal limits | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0217 | syn_file.at:1694: 217. FILE ... FROM literal | checker |  | CANDIDATE_CHECK_REJECT |
| 0218 | syn_file.at:1753: 218. WRITE / REWRITE on REPORT files | checker |  | CANDIDATE_CHECK_REJECT |
| 0219 | syn_file.at:1790: 219. SELECT without fd-name | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0220 | syn_file.at:1811: 220. Undeclared FILE-ID variable | checker |  | CANDIDATE_CHECK_REJECT |
| 0222 | syn_file.at:1864: 222. Undeclared ASSIGN variable | grammar | cobc-rs: unsupported: verb IDENTIFICATION | CANDIDATE_CHECK_REJECT |
| 0223 | syn_file.at:1943: 223. ACCESS RANDOM with ORG SEQUENTIAL | checker |  | CANDIDATE_CHECK_REJECT |
| 0228 | syn_file.at:2125: 228. SELECT/OPEN syntax extensions | checker |  | CANDIDATE_CHECK_REJECT |
| 0230 | syn_file.at:2258: 230. Invalid file name in SELECT | checker |  | CANDIDATE_CHECK_REJECT |
| 0231 | syn_reportwriter.at:23: 231. REPORT error/warning | grammar | cobc-rs: unsupported: verb END-OF-FILE-SWITCH | CANDIDATE_CHECK_REJECT |
| 0232 | syn_reportwriter.at:124: 232. REPORT not positive integers in COL / LINE PLUS | checker |  | CANDIDATE_CHECK_REJECT |
| 0233 | syn_reportwriter.at:177: 233. Missing PICTURE for SOURCE | checker |  | CANDIDATE_CHECK_REJECT |
| 0234 | syn_reportwriter.at:218: 234. Missing DETAIL line | checker |  | CANDIDATE_CHECK_REJECT |
| 0235 | syn_reportwriter.at:261: 235. REPORT LINE PLUS ZERO | checker |  | CANDIDATE_CHECK_REJECT |
| 0236 | syn_reportwriter.at:311: 236. Incorrect REPORT NAME | grammar | cobc-rs: unsupported: verb END-OF-FILE-SWITCH | CANDIDATE_CHECK_REJECT |
| 0237 | syn_reportwriter.at:428: 237. REPORT with PLUS RIGHT/CENTER | grammar | cobc-rs: unsupported: verb A001-LOOP | CANDIDATE_CHECK_REJECT |
| 0238 | syn_reportwriter.at:521: 238. PAGE LIMITS clause | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0239 | syn_reportwriter.at:556: 239. Report FD without period | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0241 | syn_reportwriter.at:626: 241. Incorrect USAGE clause | checker |  | CANDIDATE_CHECK_REJECT |
| 0242 | syn_refmod.at:26: 242. valid reference-modification | checker |  | CANDIDATE_CHECK_REJECT |
| 0243 | syn_refmod.at:55: 243. invalid reference-modification | checker |  | CANDIDATE_CHECK_REJECT |
| 0244 | syn_refmod.at:79: 244. Static out of bounds | checker |  | CANDIDATE_CHECK_REJECT |
| 0245 | syn_refmod.at:123: 245. constant-folding out of bounds | data-layout | cobc-rs: unsupported: PIC X(VAR-LEN): BadRepeat | CANDIDATE_CHECK_REJECT |
| 0247 | syn_misc.at:23: 247. ambiguous AND/OR | checker |  | CANDIDATE_CHECK_REJECT |
| 0248 | syn_misc.at:54: 248. warn constant expressions | checker |  | CANDIDATE_CHECK_REJECT |
| 0249 | syn_misc.at:120: 249. warn literal size | checker |  | CANDIDATE_CHECK_REJECT |
| 0250 | syn_misc.at:388: 250. warn literal size in constant expr. (level 88) | checker |  | CANDIDATE_CHECK_REJECT |
| 0251 | syn_misc.at:438: 251. Invalid conditional expression (1) | checker |  | CANDIDATE_CHECK_REJECT |
| 0252 | syn_misc.at:550: 252. Invalid conditional expression (2) | checker |  | CANDIDATE_CHECK_REJECT |
| 0253 | syn_misc.at:619: 253. Invalid conditional expression (3) | checker |  | CANDIDATE_CHECK_REJECT |
| 0255 | syn_misc.at:716: 255. missing headers | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0256 | syn_misc.at:773: 256. one line program | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0257 | syn_misc.at:794: 257. empty program | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0258 | syn_misc.at:842: 258. INITIALIZE constant | checker |  | CANDIDATE_CHECK_REJECT |
| 0259 | syn_misc.at:869: 259. CLASS duplicate values | checker |  | CANDIDATE_CHECK_REJECT |
| 0260 | syn_misc.at:902: 260. INSPECT invalid size | checker | cobc-rs: unsupported: expected data name after a level number | CANDIDATE_CHECK_REJECT |
| 0261 | syn_misc.at:943: 261. INSPECT invalid target | checker |  | CANDIDATE_CHECK_REJECT |
| 0262 | syn_misc.at:966: 262. INSPECT missing keyword | checker |  | CANDIDATE_CHECK_REJECT |
| 0263 | syn_misc.at:987: 263. INSPECT repeated keywords | checker |  | CANDIDATE_CHECK_REJECT |
| 0264 | syn_misc.at:1023: 264. INSPECT incomplete clause | checker |  | CANDIDATE_CHECK_REJECT |
| 0265 | syn_misc.at:1045: 265. INSPECT multiple BEFORE/AFTER clauses | checker |  | CANDIDATE_CHECK_REJECT |
| 0267 | syn_misc.at:1104: 267. maximum data size | data-layout | cobc-rs: unsupported: PIC X(9999999999): BadRepeat | CANDIDATE_CHECK_REJECT |
| 0268 | syn_misc.at:1152: 268. unreachable statement | grammar | cobc-rs: unsupported: verb USE | CANDIDATE_CHECK_REJECT |
| 0269 | syn_misc.at:1202: 269. CRT STATUS | checker |  | CANDIDATE_CHECK_REJECT |
| 0270 | syn_misc.at:1244: 270. SPECIAL-NAMES clause | checker |  | CANDIDATE_CHECK_REJECT |
| 0271 | syn_misc.at:1345: 271. CURRENCY SIGN | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0272 | syn_misc.at:1457: 272. SWITCHES | checker |  | CANDIDATE_CHECK_REJECT |
| 0273 | syn_misc.at:1583: 273. unexpected mnemonic-name location | checker |  | CANDIDATE_CHECK_REJECT |
| 0274 | syn_misc.at:1610: 274. wrong device for mnemonic-name | checker |  | CANDIDATE_CHECK_REJECT |
| 0275 | syn_misc.at:1635: 275. missing mnemonic-name declaration | checker |  | CANDIDATE_CHECK_REJECT |
| 0282 | syn_misc.at:1812: 282. source text after program-text area | grammar | cobc-rs: unsupported: verb COMMENT | CANDIDATE_CHECK_REJECT |
| 0283 | syn_misc.at:1833: 283. line overflow in fixed-form / free-form | checker |  | CANDIDATE_CHECK_REJECT |
| 0284 | syn_misc.at:1891: 284. missing newline in fixed-form / free-form | checker |  | CANDIDATE_CHECK_REJECT |
| 0285 | syn_misc.at:1925: 285. continuation of COBOL words | checker |  | CANDIDATE_CHECK_REJECT |
| 0286 | syn_misc.at:1947: 286. line and floating comments | grammar | cobc-rs: unsupported: verb | | CANDIDATE_CHECK_REJECT |
| 0288 | syn_misc.at:2264: 288. Segmentation Module | grammar | cobc-rs: unsupported: verb DEC-1 | CANDIDATE_CHECK_REJECT |
| 0290 | syn_misc.at:2382: 290. ACUCOBOL USAGE FLOAT / DOUBLE | data-layout | cobc-rs: unsupported: unrecognized USAGE FLOAT | CANDIDATE_CHECK_REJECT |
| 0291 | syn_misc.at:2411: 291. ACUCOBOL USAGE HANDLE | grammar | cobc-rs: unsupported: verb THREAD | CANDIDATE_CHECK_REJECT |
| 0292 | syn_misc.at:2518: 292. ACUCOBOL WINDOW statements | checker |  | CANDIDATE_CHECK_REJECT |
| 0293 | syn_misc.at:2603: 293. ACUCOBOL GRAPHICAL controls | data-layout | cobc-rs: unsupported: unsupported level number SCREEN | CANDIDATE_CHECK_REJECT |
| 0294 | syn_misc.at:2681: 294. DISPLAY MESSAGE BOX | checker |  | CANDIDATE_CHECK_REJECT |
| 0295 | syn_misc.at:2724: 295. DISPLAY OMITTED | checker |  | CANDIDATE_CHECK_REJECT |
| 0296 | syn_misc.at:2745: 296. CGI: EXTERNAL-FORM | checker |  | CANDIDATE_CHECK_REJECT |
| 0300 | syn_misc.at:2895: 300. complete specified word list | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0301 | syn_misc.at:2915: 301. ANY LENGTH item as formal parameter | checker |  | CANDIDATE_CHECK_REJECT |
| 0302 | syn_misc.at:2950: 302. ANY LENGTH item as BY VALUE formal parameter | checker |  | CANDIDATE_CHECK_REJECT |
| 0303 | syn_misc.at:2973: 303. swapped SOURCE- and OBJECT-COMPUTER | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0304 | syn_misc.at:2995: 304. CONF. SECTION paragraphs in wrong order | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0305 | syn_misc.at:3067: 305. NOT ON EXCEPTION with STATIC CALL convention | checker |  | CANDIDATE_CHECK_REJECT |
| 0306 | syn_misc.at:3115: 306. NOT ON EXCEPTION phrases before ON EXCEPTION | grammar | cobc-rs: unsupported: verb END-OF-PAGE | CANDIDATE_CHECK_REJECT |
| 0307 | syn_misc.at:3179: 307. wrong dialect hints | checker |  | CANDIDATE_CHECK_REJECT |
| 0308 | syn_misc.at:3204: 308. redundant periods | checker | cobc-rs: COPY expansion failed (fail closed): copybook 'A' not found | CANDIDATE_CHECK_REJECT |
| 0309 | syn_misc.at:3234: 309. missing periods | checker |  | CANDIDATE_CHECK_REJECT |
| 0311 | syn_misc.at:3363: 311. statement in Area A | grammar | cobc-rs: unsupported: verb SEC-1 | CANDIDATE_CHECK_REJECT |
| 0313 | syn_misc.at:3473: 313. IF-ELSE statement list with invalid syntax | checker |  | CANDIDATE_CHECK_REJECT |
| 0314 | syn_misc.at:3506: 314. EVALUATE statement with invalid syntax | checker |  | CANDIDATE_CHECK_REJECT |
| 0315 | syn_misc.at:3555: 315. COBOL-WORDS directive | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0316 | syn_misc.at:3629: 316. MF reserved word directives | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0317 | syn_misc.at:3691: 317. TURN directive | checker |  | CANDIDATE_CHECK_REJECT |
| 0318 | syn_misc.at:3724: 318. STRING / UNSTRING with invalid syntax | checker |  | CANDIDATE_CHECK_REJECT |
| 0319 | syn_misc.at:3789: 319. STRING / UNSTRING POINTER clause | data-layout | cobc-rs: unsupported: PIC VPP99: ScalingPDeferred | CANDIDATE_CHECK_REJECT |
| 0320 | syn_misc.at:3864: 320. STRING with non-DISPLAY | checker |  | CANDIDATE_CHECK_REJECT |
| 0321 | syn_misc.at:3903: 321. UNSTRING COUNT clause | data-layout | cobc-rs: unsupported: PIC VPP99: ScalingPDeferred | CANDIDATE_CHECK_REJECT |
| 0324 | syn_misc.at:4115: 324. invalid INSPECT/TRANSFORM operands | checker |  | CANDIDATE_CHECK_REJECT |
| 0325 | syn_misc.at:4167: 325. SIGN clause checks | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0326 | syn_misc.at:4192: 326. conflicting entry conventions | checker |  | CANDIDATE_CHECK_REJECT |
| 0327 | syn_misc.at:4261: 327. conflicting call conventions | checker |  | CANDIDATE_CHECK_REJECT |
| 0328 | syn_misc.at:4290: 328. dangling LINKAGE items | checker |  | CANDIDATE_CHECK_REJECT |
| 0329 | syn_misc.at:4346: 329. duplicate PROCEDURE DIVISION/ENTRY USING items | checker |  | CANDIDATE_CHECK_REJECT |
| 0330 | syn_misc.at:4367: 330. duplicate CALL USING BY REFERENCE items | checker |  | CANDIDATE_CHECK_REJECT |
| 0331 | syn_misc.at:4393: 331. ADD / SUBTRACT TABLE | checker |  | CANDIDATE_CHECK_REJECT |
| 0332 | syn_misc.at:4436: 332. USE FOR DEBUGGING invalid ref-mod / subscripts | grammar | cobc-rs: unsupported: verb USE | CANDIDATE_CHECK_REJECT |
| 0333 | syn_misc.at:4481: 333. USE FOR DEBUGGING duplicate targets | grammar | cobc-rs: unsupported: verb USE | CANDIDATE_CHECK_REJECT |
| 0334 | syn_misc.at:4534: 334. USE FOR DEBUGGING implicit statements | grammar | cobc-rs: unsupported: verb USE | CANDIDATE_CHECK_REJECT |
| 0335 | syn_misc.at:4582: 335. USE FOR DEBUGGING syntax-checks (1) | grammar | cobc-rs: unsupported: verb USE | CANDIDATE_CHECK_REJECT |
| 0338 | syn_misc.at:4776: 338. whitespace handling | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0339 | syn_misc.at:4864: 339. STOP identifier | checker |  | CANDIDATE_CHECK_REJECT |
| 0340 | syn_misc.at:4890: 340. 01 CONSTANT | checker |  | CANDIDATE_CHECK_REJECT |
| 0341 | syn_misc.at:4938: 341. 78 VALUE | checker |  | CANDIDATE_CHECK_REJECT |
| 0342 | syn_misc.at:4985: 342. level 78 NEXT / START OF | checker |  | CANDIDATE_CHECK_REJECT |
| 0343 | syn_misc.at:5044: 343. SYMBOLIC CONSTANT | checker |  | CANDIDATE_CHECK_REJECT |
| 0344 | syn_misc.at:5093: 344. Constant Expressions (1) | checker |  | CANDIDATE_CHECK_REJECT |
| 0345 | syn_misc.at:5218: 345. Constant Expressions (2) | data-layout | cobc-rs: unsupported: PIC X(CONST1): BadRepeat | CANDIDATE_CHECK_REJECT |
| 0346 | syn_misc.at:5274: 346. Constant Expressions (3) | grammar | cobc-rs: unsupported: verb NOTDEFINED | CANDIDATE_CHECK_REJECT |
| 0347 | syn_misc.at:5365: 347. Constant Expressions (4) | checker |  | CANDIDATE_CHECK_REJECT |
| 0351 | syn_misc.at:5629: 351. CONSTANT LENGTH / BYTE-LENGTH | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0352 | syn_misc.at:5653: 352. ANY LENGTH/NUMERIC with incorrect PIC | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0353 | syn_misc.at:5699: 353. VOLATILE clause | checker |  | CANDIDATE_CHECK_REJECT |
| 0354 | syn_misc.at:5743: 354. SET SOURCEFORMAT syntax checks | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0355 | syn_misc.at:5776: 355. WHEN-COMPILED register in dialect | checker |  | CANDIDATE_CHECK_REJECT |
| 0356 | syn_misc.at:5802: 356. LIN / COL register | checker |  | CANDIDATE_CHECK_REJECT |
| 0358 | syn_misc.at:5862: 358. @OPTIONS parsing | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0359 | syn_misc.at:5906: 359. PROCESS / CBL parsing | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0360 | syn_misc.at:5971: 360. *CONTROL / *CBL parsing | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0361 | syn_misc.at:6004: 361. system routines with wrong number of parameters | checker |  | CANDIDATE_CHECK_REJECT |
| 0362 | syn_misc.at:6033: 362. invalid use of condition-name | checker |  | CANDIDATE_CHECK_REJECT |
| 0363 | syn_misc.at:6097: 363. XML GENERATE syntax checks | data-layout | cobc-rs: unsupported: unrecognized USAGE BIT | CANDIDATE_CHECK_REJECT |
| 0364 | "syn_misc.at:6315: 364. BASED clause, ALLOCATE / FREE statements" | data-layout | cobc-rs: unsupported: unsupported level number SCREEN-STORAGE | CANDIDATE_CHECK_REJECT |
| 0365 | syn_misc.at:6396: 365. CONTINUE statement | grammar | cobc-rs: unsupported: verb AFTER | CANDIDATE_CHECK_REJECT |
| 0366 | syn_misc.at:6440: 366. conflict markers | data-layout | cobc-rs: unsupported: unsupported level number HEAD | CANDIDATE_CHECK_REJECT |
| 0367 | syn_misc.at:6499: 367. SORT syntax | checker |  | CANDIDATE_CHECK_REJECT |
| 0368 | syn_misc.at:6553: 368. OSVS I/O extensions | checker |  | CANDIDATE_CHECK_REJECT |
| 0370 | syn_misc.at:6646: 370. SEARCH ALL checks | checker |  | CANDIDATE_CHECK_REJECT |
| 0371 | syn_misc.at:6766: 371. Invalid parentheses around condition | checker |  | CANDIDATE_CHECK_REJECT |
| 0372 | syn_misc.at:6789: 372. DISPLAY directive (1) | checker |  | CANDIDATE_CHECK_REJECT |
| 0373 | syn_misc.at:6812: 373. DISPLAY directive (2) | checker |  | CANDIDATE_CHECK_REJECT |
| 0374 | syn_misc.at:6832: 374. DISPLAY directive (3) | checker |  | CANDIDATE_CHECK_REJECT |
| 0375 | syn_misc.at:6851: 375. SET CONSTANT directive | data-layout | cobc-rs: unsupported: unsupported level number $SET | CANDIDATE_CHECK_REJECT |
| 0376 | syn_misc.at:6950: 376. conditional / define directives (1) | checker |  | CANDIDATE_CHECK_REJECT |
| 0377 | syn_misc.at:6975: 377. conditional / define directives (2) | checker |  | CANDIDATE_CHECK_REJECT |
| 0378 | syn_misc.at:7003: 378. conditional / define directives (3) | checker |  | CANDIDATE_CHECK_REJECT |
| 0380 | syn_misc.at:7055: 380. error handling in conditional directives | checker |  | CANDIDATE_CHECK_REJECT |
| 0384 | syn_misc.at:7197: 384. Invalid PERFORM statement | checker |  | CANDIDATE_CHECK_REJECT |
| 0385 | syn_misc.at:7228: 385. PERFORM THRU syntax checks | grammar | cobc-rs: unsupported: verb SUB1 | CANDIDATE_CHECK_REJECT |
| 0386 | syn_misc.at:7276: 386. VALIDATE parsing | data-layout | cobc-rs: unsupported: unrecognized USAGE BIT | CANDIDATE_CHECK_REJECT |
| 0392 | syn_misc.at:7894: 392. CONTROL DIVISION & AREACHECK | checker |  | CANDIDATE_CHECK_REJECT |
| 0393 | syn_misc.at:7930: 393. PICTURE L | checker |  | CANDIDATE_CHECK_REJECT |
| 0394 | syn_misc.at:8025: 394. AREACHECK / NOAREACHECK directives | grammar | cobc-rs: unsupported: verb $SET | CANDIDATE_CHECK_REJECT |
| 0395 | syn_misc.at:8065: 395. AREACHECK / NOAREACHECK directives (2) | data-layout | cobc-rs: unsupported: unsupported level number $SET | CANDIDATE_CHECK_REJECT |
| 0396 | syn_misc.at:8117: 396. optional dots | grammar | cobc-rs: unsupported: verb P | CANDIDATE_CHECK_REJECT |
| 0397 | syn_misc.at:8172: 397. optional dots before PROCEDURE DIVISION | checker |  | CANDIDATE_CHECK_REJECT |
| 0398 | syn_misc.at:8209: 398. AREACHECK | checker |  | CANDIDATE_CHECK_REJECT |
| 0399 | syn_misc.at:8249: 399. autodetect format | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0401 | syn_move.at:38: 401. MOVE SPACE TO numeric or numeric-edited item | checker |  | CANDIDATE_CHECK_REJECT |
| 0402 | syn_move.at:64: 402. MOVE ZERO TO alphabetic item | checker |  | CANDIDATE_CHECK_REJECT |
| 0403 | syn_move.at:90: 403. MOVE alphabetic TO x | data-layout | cobc-rs: unsupported: PIC BX: UnsupportedSymbol('X') | CANDIDATE_CHECK_REJECT |
| 0404 | syn_move.at:124: 404. MOVE alphanumeric TO x | data-layout | cobc-rs: unsupported: PIC BX: UnsupportedSymbol('X') | CANDIDATE_CHECK_REJECT |
| 0405 | syn_move.at:155: 405. MOVE alphanumeric-edited TO x | data-layout | cobc-rs: unsupported: PIC BX: UnsupportedSymbol('X') | CANDIDATE_CHECK_REJECT |
| 0406 | syn_move.at:189: 406. MOVE numeric (integer) TO x | data-layout | cobc-rs: unsupported: PIC BX: UnsupportedSymbol('X') | CANDIDATE_CHECK_REJECT |
| 0407 | syn_move.at:222: 407. MOVE numeric (non-integer) TO x | data-layout | cobc-rs: unsupported: PIC BX: UnsupportedSymbol('X') | CANDIDATE_CHECK_REJECT |
| 0408 | syn_move.at:258: 408. MOVE numeric-edited TO x | data-layout | cobc-rs: unsupported: PIC BX: UnsupportedSymbol('X') | CANDIDATE_CHECK_REJECT |
| 0409 | syn_move.at:293: 409. MOVE national TO x | data-layout | cobc-rs: unsupported: PIC N: UnsupportedSymbol('N') | CANDIDATE_CHECK_REJECT |
| 0410 | syn_move.at:330: 410. MOVE national-edited TO x | data-layout | cobc-rs: unsupported: PIC 0N: UnsupportedSymbol('N') | CANDIDATE_CHECK_REJECT |
| 0411 | syn_move.at:374: 411. CORRESPONDING - Operands must be groups | checker |  | CANDIDATE_CHECK_REJECT |
| 0412 | syn_move.at:404: 412. CORRESPONDING - Target has no matching items | checker |  | CANDIDATE_CHECK_REJECT |
| 0413 | syn_move.at:430: 413. MOVE to erroneous field | checker |  | CANDIDATE_CHECK_REJECT |
| 0414 | syn_move.at:453: 414. Overlapping MOVE | checker |  | CANDIDATE_CHECK_REJECT |
| 0415 | syn_move.at:552: 415. invalid source for MOVE | checker |  | CANDIDATE_CHECK_REJECT |
| 0416 | syn_move.at:582: 416. invalid target for MOVE | checker |  | CANDIDATE_CHECK_REJECT |
| 0417 | syn_move.at:618: 417. SET error | checker |  | CANDIDATE_CHECK_REJECT |
| 0419 | syn_multiply.at:28: 419. Category check of Format 1 | checker |  | CANDIDATE_CHECK_REJECT |
| 0420 | syn_multiply.at:64: 420. Category check of Format 2 | checker |  | CANDIDATE_CHECK_REJECT |
| 0421 | syn_multiply.at:102: 421. Category check of literals | checker |  | CANDIDATE_CHECK_REJECT |
| 0422 | syn_screen.at:24: 422. Flexible ACCEPT/DISPLAY syntax | data-layout | cobc-rs: unsupported: unsupported level number SCREEN | CANDIDATE_CHECK_REJECT |
| 0423 | syn_screen.at:92: 423. Duplicate ACCEPT/DISPLAY clauses | checker |  | CANDIDATE_CHECK_REJECT |
| 0424 | syn_screen.at:121: 424. AT clause | checker |  | CANDIDATE_CHECK_REJECT |
| 0426 | syn_screen.at:221: 426. FROM clause | data-layout | cobc-rs: unsupported: unsupported level number SCREEN | CANDIDATE_CHECK_REJECT |
| 0427 | syn_screen.at:250: 427. Incorrect USAGE clause | checker |  | CANDIDATE_CHECK_REJECT |
| 0428 | syn_screen.at:283: 428. SCREEN SECTION clause numbers | checker |  | CANDIDATE_CHECK_REJECT |
| 0429 | syn_screen.at:315: 429. Screen clauses | data-layout | cobc-rs: unsupported: unsupported level number SCREEN | CANDIDATE_CHECK_REJECT |
| 0430 | syn_screen.at:341: 430. ACCEPT ON EXCEPTION/ESCAPE | grammar | cobc-rs: unsupported: verb NOT | CANDIDATE_CHECK_REJECT |
| 0431 | syn_screen.at:371: 431. Referencing 88-level | data-layout | cobc-rs: unsupported: unsupported level number SCREEN | CANDIDATE_CHECK_REJECT |
| 0432 | syn_screen.at:402: 432. Conflicting screen clauses | data-layout | cobc-rs: unsupported: unsupported level number SCREEN | CANDIDATE_CHECK_REJECT |
| 0433 | syn_screen.at:473: 433. Redundant screen clauses | data-layout | cobc-rs: unsupported: unsupported level number SCREEN | CANDIDATE_CHECK_REJECT |
| 0434 | syn_screen.at:506: 434. Screen item OCCURS w-/wo relative LINE/COL | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0435 | syn_screen.at:566: 435. VALUE clause missing | data-layout | cobc-rs: unsupported: unsupported level number SCREEN | CANDIDATE_CHECK_REJECT |
| 0436 | syn_screen.at:592: 436. FULL on numeric item | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0438 | syn_screen.at:774: 438. MS-COBOL position-spec | checker |  | CANDIDATE_CHECK_REJECT |
| 0439 | syn_screen.at:822: 439. Screen with invalid FROM clause | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0440 | syn_screen.at:867: 440. DISPLAY WITH CONVERSION | checker |  | CANDIDATE_CHECK_REJECT |
| 0441 | syn_set.at:24: 441. SET ADDRESS OF item | checker |  | CANDIDATE_CHECK_REJECT |
| 0442 | syn_set.at:53: 442. SET item TO 88-level | checker |  | CANDIDATE_CHECK_REJECT |
| 0443 | syn_functions.at:22: 443. ANY LENGTH / NUMERIC as function RETURNING item | checker | cobc-rs: unsupported: no PROGRAM-ID | CANDIDATE_CHECK_REJECT |
| 0446 | syn_functions.at:135: 446. Redundant REPOSITORY entries | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0447 | syn_functions.at:174: 447. Missing prototype/definition | checker | cobc-rs: unsupported: no PROGRAM-ID | CANDIDATE_CHECK_REJECT |
| 0448 | syn_functions.at:205: 448. Empty function | checker | cobc-rs: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0449 | syn_functions.at:232: 449. Function definition inside program | grammar | cobc-rs: unsupported: verb IDENTIFICATION | CANDIDATE_CHECK_REJECT |
| 0450 | syn_functions.at:255: 450. Intrinsic functions: dialect | checker |  | CANDIDATE_CHECK_REJECT |
| 0452 | syn_functions.at:324: 452. Intrinsic functions: number of arguments | checker |  | CANDIDATE_CHECK_REJECT |
| 0453 | syn_functions.at:364: 453. Intrinsic functions: reference modification | checker |  | CANDIDATE_CHECK_REJECT |
| 0454 | syn_functions.at:410: 454. Intrinsic functions: argument type | checker |  | CANDIDATE_CHECK_REJECT |
| 0455 | syn_functions.at:433: 455. invalid formatted date/time args | checker |  | CANDIDATE_CHECK_REJECT |
| 0456 | syn_functions.at:514: 456. invalid formats w/ DECIMAL-POINT IS COMMA | checker |  | CANDIDATE_CHECK_REJECT |
| 0457 | syn_functions.at:544: 457. Specified offset and SYSTEM-OFFSET | checker |  | CANDIDATE_CHECK_REJECT |
| 0458 | syn_functions.at:568: 458. FUNCTION LENGTH / BYTE-LENGTH | checker |  | CANDIDATE_CHECK_REJECT |
| 0459 | syn_literals.at:25: 459. continuation Indicator - too many lines | checker |  | CANDIDATE_CHECK_REJECT |
| 0460 | syn_literals.at:583: 460. literal too long | checker |  | CANDIDATE_CHECK_REJECT |
| 0462 | syn_literals.at:998: 462. floating-point literals | grammar | "cobc-rs: unsupported: verb E1," | CANDIDATE_CHECK_REJECT |
| 0463 | syn_literals.at:1105: 463. X literals | checker |  | CANDIDATE_CHECK_REJECT |
| 0464 | syn_literals.at:1140: 464. national literals | checker |  | CANDIDATE_CHECK_REJECT |
| 0465 | syn_literals.at:1178: 465. NX literals | checker |  | CANDIDATE_CHECK_REJECT |
| 0466 | syn_literals.at:1216: 466. binary literals | checker |  | CANDIDATE_CHECK_REJECT |
| 0467 | syn_literals.at:1252: 467. binary-hexadecimal literals | checker |  | CANDIDATE_CHECK_REJECT |
| 0469 | syn_literals.at:1317: 469. ACUCOBOL literals | checker |  | CANDIDATE_CHECK_REJECT |
| 0471 | syn_literals.at:1412: 471. zero-length literals | data-layout | cobc-rs: unsupported: PIC N: UnsupportedSymbol('N') | CANDIDATE_CHECK_REJECT |
| 0472 | syn_literals.at:1472: 472. long literal in error message | checker |  | CANDIDATE_CHECK_REJECT |
| 0473 | syn_literals.at:1504: 473. literal missing terminating character | checker |  | CANDIDATE_CHECK_REJECT |
| 0512 | "run_fundamental.at:72: 512. DISPLAY literals, DECIMAL-POINT is COMMA" | grammar | "cobrun: unsupported: not a numeric literal: 1,23E0" | CANDIDATE_CHECK_REJECT |
| 0513 | run_fundamental.at:105: 513. Hexadecimal literal | checker | "cobrun: unsupported: CALL ""dump"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0518 | run_fundamental.at:345: 518. MOVE to edited item (3) | data-layout | cobrun: unsupported: PIC 0XXXXXX: UnsupportedSymbol('X') | CANDIDATE_CHECK_REJECT |
| 0519 | run_fundamental.at:450: 519. MOVE to item with simple and floating insertion | data-layout | cobrun: unsupported: unsupported level number -- | CANDIDATE_CHECK_REJECT |
| 0521 | run_fundamental.at:530: 521. MOVE integer literal to alphanumeric | checker |  | CANDIDATE_CHECK_REJECT |
| 0523 | run_fundamental.at:620: 523. equality of FLOAT-SHORT / FLOAT-LONG | grammar | cobrun: unsupported: not a numeric literal: FAILED | CANDIDATE_CHECK_REJECT |
| 0524 | run_fundamental.at:743: 524. equality of FLOAT-SHORT / FLOAT-EXTENDED | grammar | cobrun: unsupported: not a numeric literal: ORT | CANDIDATE_CHECK_REJECT |
| 0525 | run_fundamental.at:874: 525. Overlapping MOVE (GnuCOBOL) | checker |  | CANDIDATE_CHECK_REJECT |
| 0530 | run_fundamental.at:1135: 530. GLOBAL at same level | grammar | cobrun: unsupported: not a numeric literal: X | CANDIDATE_CHECK_REJECT |
| 0531 | run_fundamental.at:1184: 531. GLOBAL at lower level | grammar | cobrun: unsupported: not a numeric literal: X | CANDIDATE_CHECK_REJECT |
| 0532 | run_fundamental.at:1233: 532. GLOBAL CONSTANT | grammar | cobrun: unsupported: not a numeric literal: GLOB-PATH2 | CANDIDATE_CHECK_REJECT |
| 0533 | run_fundamental.at:1317: 533. GLOBAL identifiers from ENVIRONMENT DIVISION | data-layout | "cobrun: unsupported: PIC 9.9999,99Y: UnsupportedSymbol('Y')" | CANDIDATE_CHECK_REJECT |
| 0536 | run_fundamental.at:1500: 536. Entry point visibility (1) | checker | "cobrun: unsupported: CALL ""modulepart"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0537 | run_fundamental.at:1532: 537. Entry point visibility (2) | checker | "cobrun: unsupported: CALL ""module"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0538 | run_fundamental.at:1570: 538. Contained program visibility (1) | grammar | cobrun: unsupported: not a numeric literal: X | CANDIDATE_CHECK_REJECT |
| 0539 | run_fundamental.at:1625: 539. Contained program visibility (2) | grammar | cobrun: unsupported: not a numeric literal: X | CANDIDATE_CHECK_REJECT |
| 0540 | run_fundamental.at:1678: 540. Contained program visibility (3) | grammar | cobrun: unsupported: not a numeric literal: X | CANDIDATE_CHECK_REJECT |
| 0543 | run_fundamental.at:1872: 543. CALL program-pointer | checker | "cobrun: unsupported: CALL ""PROG-PTR"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0544 | run_fundamental.at:1968: 544. CALL/CANCEL/SET ADDRESS program-prototype-name | grammar | cobrun: unsupported: not a numeric literal: X | CANDIDATE_CHECK_REJECT |
| 0555 | run_fundamental.at:2459: 555. Context sensitive words (2) | checker | cobrun: unsupported: ACCEPT FROM DATE/TIME requires a pinned COB_CURRENT_DATE (the live clock is a non-claim) | CANDIDATE_CHECK_REJECT |
| 0556 | run_fundamental.at:2483: 556. Context sensitive words (3) | checker | cobrun: unsupported: ACCEPT FROM DATE/TIME requires a pinned COB_CURRENT_DATE (the live clock is a non-claim) | CANDIDATE_CHECK_REJECT |
| 0570 | run_fundamental.at:3108: 570. Numeric operations (2) DISPLAY | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 0571 | run_fundamental.at:3344: 571. Numeric operations (3) PACKED-DECIMAL | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 0572 | run_fundamental.at:3648: 572. Numeric operations (4) BINARY | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 0573 | run_fundamental.at:3883: 573. Numeric operations (5) COMP-5 | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 0574 | run_fundamental.at:4118: 574. Numeric operations (6) | checker | "cobrun: unsupported: CALL ""dump"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0576 | run_fundamental.at:4468: 576. Numeric operations (8) | checker |  | CANDIDATE_CHECK_REJECT |
| 0578 | run_fundamental.at:4566: 578. ADD CORRESPONDING no match | checker |  | CANDIDATE_CHECK_REJECT |
| 0579 | run_fundamental.at:4616: 579. SYNC in OCCURS | grammar | cobrun: unsupported: not a numeric literal: FUNCTION | CANDIDATE_CHECK_REJECT |
| 0581 | run_fundamental.at:4751: 581. 88 level with FILLER | name-resolution | cobrun: undefined data name: FILLER | CANDIDATE_PARSE_REJECT |
| 0582 | run_fundamental.at:4780: 582. 88 level with FALSE IS clause | checker | cobrun: unsupported: SET MYFLD88 TO FALSE: the 88 has no `WHEN SET TO FALSE` value | CANDIDATE_CHECK_REJECT |
| 0585 | run_fundamental.at:4867: 585. DIVIDE complex | name-resolution | cobrun: undefined data name: RES-TAB(1) | CANDIDATE_PARSE_REJECT |
| 0596 | run_fundamental.at:5331: 596. USE FOR DEBUGGING (COB_SET_DEBUG switched) | name-resolution | cobrun: undefined data name: ENVIRONMENT | CANDIDATE_PARSE_REJECT |
| 0603 | run_fundamental.at:5682: 603. Simple Expressions with figurative constants | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 0604 | run_fundamental.at:6025: 604. Expression numeric vs. DISPLAY | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 0605 | run_fundamental.at:6080: 605. Abbreviated Expressions | checker |  | CANDIDATE_CHECK_REJECT |
| 0607 | run_fundamental.at:6234: 607. TYPEDEF application | data-layout | cobrun: unsupported: unrecognized USAGE INT | CANDIDATE_CHECK_REJECT |
| 0608 | run_fundamental.at:6286: 608. Alphanumeric VALUE longer than PIC | checker |  | CANDIDATE_CHECK_REJECT |
| 0609 | run_fundamental.at:6318: 609. DISPLAY with P fields | data-layout | cobrun: unsupported: PIC VP9: ScalingPDeferred | CANDIDATE_CHECK_REJECT |
| 0610 | run_fundamental.at:6404: 610. condition IS ZERO AND | checker |  | CANDIDATE_CHECK_REJECT |
| 0612 | run_fundamental.at:6493: 612. abbreviated conditions with multiple words operators | checker |  | CANDIDATE_CHECK_REJECT |
| 0621 | run_fundamental.at:7164: 621. MOVE misc. edited | data-layout | "cobrun: unsupported: PIC $$$$,$$9V99-: UnsupportedSymbol('V')" | CANDIDATE_CHECK_REJECT |
| 0624 | run_fundamental.at:9066: 624. SPECIAL-NAMES CLASS | semantic-check | cobrun: unsupported: condition: unrecognized relational operator (expected = > < >= <= <> GREATER LESS EQUAL) | CANDIDATE_CHECK_REJECT |
| 0630 | run_subscripts.at:211: 630. Subscript by arithmetic expression | checker | cobrun: unsupported: subscript '(3 + 1) / 2' is not an integer | CANDIDATE_CHECK_REJECT |
| 0633 | run_subscripts.at:351: 633. enable / disable subscript check | grammar | cobrun: unsupported: not a numeric literal: FUNCTION | CANDIDATE_CHECK_REJECT |
| 0634 | run_subscripts.at:396: 634. enable / disable subscript check with ODO | name-resolution | cobrun: undefined data name: NOT | CANDIDATE_PARSE_REJECT |
| 0636 | run_subscripts.at:506: 636. SSRANGE and NOSSRANGE directives | checker | cobrun: unsupported: SET: expected `TO` or `UP|DOWN BY` (cobc rejects a SET with neither) | CANDIDATE_CHECK_REJECT |
| 0637 | run_subscripts.at:542: 637. CALL with OCCURS DEPENDING ON | grammar | cobrun: unsupported: not a numeric literal: PARM-STR | CANDIDATE_CHECK_REJECT |
| 0641 | run_refmod.at:118: 641. Offset overflow | checker | cobrun: unsupported: expected data name after a level number | CANDIDATE_CHECK_REJECT |
| 0649 | run_accept.at:29: 649. ACCEPT OMITTED (simple) | checker | cobrun: unsupported: ACCEPT FROM terminal/console: interactive input is a runtime non-claim (no deterministic oracle); the wired sources are DATE/DAY/TIME/DAY-OF-WEEK/ENVIRONMENT | CANDIDATE_CHECK_REJECT |
| 0650 | run_accept.at:51: 650. ACCEPT FROM TIME / DATE / DAY / DAY-OF-WEEK (1) | checker | cobrun: unsupported: ACCEPT FROM DATE/TIME requires a pinned COB_CURRENT_DATE (the live clock is a non-claim) | CANDIDATE_CHECK_REJECT |
| 0652 | run_accept.at:283: 652. ACCEPT DATE / DAY and intrinsic functions (1) | checker | cobrun: unsupported: ACCEPT FROM DATE/TIME requires a pinned COB_CURRENT_DATE (the live clock is a non-claim) | CANDIDATE_CHECK_REJECT |
| 0654 | run_accept.at:367: 654. ACCEPT OMITTED (SCREEN) | checker | cobrun: unsupported: ACCEPT FROM terminal/console: interactive input is a runtime non-claim (no deterministic oracle); the wired sources are DATE/DAY/TIME/DAY-OF-WEEK/ENVIRONMENT | CANDIDATE_CHECK_REJECT |
| 0657 | run_initialize.at:90: 657. INITIALIZE OCCURS with SIGN LEADING / TRAILING | name-resolution | "cobrun: undefined data name: X(1)," | CANDIDATE_PARSE_REJECT |
| 0664 | run_initialize.at:442: 664. INITIALIZE with FILLER | name-resolution | cobrun: undefined data name: MY-FILLER(2:3) | CANDIDATE_PARSE_REJECT |
| 0666 | run_initialize.at:560: 666. INITIALIZE with reference-modification | name-resolution | cobrun: undefined data name: MY-FLD(1:2) | CANDIDATE_PARSE_REJECT |
| 0667 | run_initialize.at:596: 667. INITIALIZE big table with VALUE | grammar | cobrun: unsupported: not a numeric literal: ALL | CANDIDATE_CHECK_REJECT |
| 0669 | run_misc.at:23: 669. Comma separator without space | grammar | "cobrun: unsupported: not a numeric literal: 1,1,1" | CANDIDATE_CHECK_REJECT |
| 0674 | run_misc.at:156: 674. DECIMAL-POINT is COMMA (5) | checker | cobrun: unsupported: COMPUTE without '=' | CANDIDATE_CHECK_REJECT |
| 0677 | run_misc.at:268: 677. LOCAL-STORAGE (1) | data-layout | cobrun: unsupported: unsupported level number LOCAL-STORAGE | CANDIDATE_CHECK_REJECT |
| 0678 | run_misc.at:304: 678. LOCAL-STORAGE (2) | name-resolution | cobrun: undefined data name: LCL-X | CANDIDATE_PARSE_REJECT |
| 0679 | run_misc.at:348: 679. LOCAL-STORAGE (3) | data-layout | cobrun: unsupported: unsupported level number LOCAL-STORAGE | CANDIDATE_CHECK_REJECT |
| 0683 | run_misc.at:594: 683. MOVE to itself | checker |  | CANDIDATE_CHECK_REJECT |
| 0688 | run_misc.at:711: 688. MOVE X'00' | checker | "cobrun: unsupported: CALL ""dump"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0689 | run_misc.at:746: 689. MOVE Z'literal' | grammar | cobrun: unsupported: not a numeric literal: Z | CANDIDATE_CHECK_REJECT |
| 0690 | run_misc.at:788: 690. Floating continuation indicator | grammar | cobrun: unsupported: not a numeric literal: - | CANDIDATE_CHECK_REJECT |
| 0691 | run_misc.at:810: 691. Fixed continuation indicator | checker | cobrun: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0693 | run_misc.at:876: 693. SOURCE FIXED/FREE directives | grammar | cobrun: unsupported: verb T | CANDIDATE_CHECK_REJECT |
| 0695 | run_misc.at:953: 695. OCCURS on level 01 | grammar | cobrun: unsupported: not a numeric literal: X-ALL | CANDIDATE_CHECK_REJECT |
| 0697 | run_misc.at:1060: 697. Index and parenthesized expression | checker | cobrun: unsupported: trailing tokens in condition at + | CANDIDATE_CHECK_REJECT |
| 0702 | run_misc.at:1232: 702. Dynamic CALL with ON EXCEPTION | checker | "cobrun: unsupported: CALL ""callee1"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0704 | run_misc.at:1304: 704. CALL m1. CALL m2. CALL m1. | grammar | cobrun: unsupported: verb IDENTIFICATION | CANDIDATE_CHECK_REJECT |
| 0709 | run_misc.at:1567: 709. Multiple calls of INITIAL program | checker | "cobrun: unsupported: CALL ""C$PARAMSIZE"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0710 | run_misc.at:1624: 710. CALL binary literal parameter/LENGTH OF | checker | "cobrun: unsupported: CALL ""dump"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0713 | run_misc.at:1726: 713. TRANSFORM statement | name-resolution | cobrun: undefined data name: MY-ASCII | CANDIDATE_PARSE_REJECT |
| 0714 | run_misc.at:1759: 714. INSPECT CONVERTING alphabet | grammar | cobrun: unsupported: not a numeric literal: X | CANDIDATE_CHECK_REJECT |
| 0716 | run_misc.at:1830: 716. INSPECT CONVERTING NULL | name-resolution | cobrun: undefined data name: NULL | CANDIDATE_PARSE_REJECT |
| 0717 | run_misc.at:1852: 717. INSPECT CONVERTING TO NULL | name-resolution | cobrun: undefined data name: NULL | CANDIDATE_PARSE_REJECT |
| 0719 | run_misc.at:1903: 719. INSPECT numeric signed | checker | cobrun: unsupported: INSPECT region clause near Str([50]) | CANDIDATE_CHECK_REJECT |
| 0729 | run_misc.at:2376: 729. PERFORM VARYING BY phrase omitted | checker | cobc-rs: unsupported: PERFORM VARYING: expected BY | CANDIDATE_CHECK_REJECT |
| 0735 | run_misc.at:2551: 735. PERFORM FOREVER / PERFORM UNTIL EXIT | semantic-check | cobrun: unsupported: condition: missing left operand | CANDIDATE_CHECK_REJECT |
| 0748 | run_misc.at:3201: 748. UNSTRING with FUNCTION / literal | name-resolution | "cobrun: undefined data name: PRM(1)," | CANDIDATE_PARSE_REJECT |
| 0749 | run_misc.at:3271: 749. SORT: table | semantic-check | cobrun: unsupported: SORT: `TBL` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0750 | run_misc.at:3305: 750. SORT: table (2) | name-resolution | "cobrun: undefined data name: TAB1-NR(K)," | CANDIDATE_PARSE_REJECT |
| 0751 | run_misc.at:3430: 751. SORT: table (3) | semantic-check | cobrun: unsupported: SORT: `ROW1` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0752 | run_misc.at:3522: 752. SORT: table (toplevel) | semantic-check | cobrun: unsupported: SORT: `TBL` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0753 | run_misc.at:3544: 753. SORT: EBCDIC table | semantic-check | cobrun: unsupported: SORT: `TBL` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0761 | run_misc.at:4085: 761. COB_PRE_LOAD | checker | "cobrun: unsupported: CALL ""callee2"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0762 | run_misc.at:4111: 762. COB_PRE_LOAD with entry points | checker | "cobrun: unsupported: CALL ""ent1"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0763 | run_misc.at:4182: 763. Lookup ENTRY from main executable | checker | cobrun: unsupported: SET PROGRAM-LINK TO <value>: target is not a numeric/index item | CANDIDATE_CHECK_REJECT |
| 0765 | run_misc.at:4249: 765. ALLOCATE / FREE with BASED item (1) | name-resolution | cobrun: undefined data name: MYFLD | CANDIDATE_PARSE_REJECT |
| 0766 | run_misc.at:4275: 766. ALLOCATE / FREE with BASED item (2) | semantic-check | cobrun: unsupported: condition: unrecognized relational operator (expected = > < >= <= <> GREATER LESS EQUAL) | CANDIDATE_CHECK_REJECT |
| 0767 | run_misc.at:4319: 767. ALLOCATE CHARACTERS INITIALIZED (TO) | name-resolution | cobrun: undefined data name: ADDRESS | CANDIDATE_PARSE_REJECT |
| 0769 | run_misc.at:4390: 769. CALL with OMITTED parameter | checker | "cobrun: unsupported: CALL ""callee"": fewer USING args than the 3 parameters" | CANDIDATE_CHECK_REJECT |
| 0770 | run_misc.at:4426: 770. direct CALL in from C w/wo error | checker | cobrun: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0771 | run_misc.at:4507: 771. direct CALL in from C w/wo error; no exit | checker | cobrun: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0772 | "run_misc.at:4641: 772. CALL in from C, cob_call_params explicitly set" | checker | cobrun: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0773 | "run_misc.at:4700: 773. CALL in from C, cob_call_params unknown" | checker | cobrun: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0774 | "run_misc.at:4753: 774. CALL C with callback, PROCEDURE DIVISION EXTERN" | checker | cobrun: unsupported: SET CB TO <value>: target is not a numeric/index item | CANDIDATE_CHECK_REJECT |
| 0775 | "run_misc.at:4823: 775. CALL C with callback, ENTRY-CONVENTION EXTERN" | checker | cobrun: unsupported: SET CB TO <value>: target is not a numeric/index item | CANDIDATE_CHECK_REJECT |
| 0776 | run_misc.at:4974: 776. CALL in from C with init missing / implicit | checker | cobrun: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0777 | run_misc.at:5022: 777. CALL STATIC C from COBOL | checker | "cobrun: unsupported: CALL ""STATIC"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0780 | run_misc.at:5157: 780. ANY LENGTH (3) | grammar | cobrun: unsupported: not a numeric literal: ALL | CANDIDATE_CHECK_REJECT |
| 0781 | run_misc.at:5200: 781. ANY LENGTH (4) | grammar | cobrun: unsupported: not a numeric literal: ALL | CANDIDATE_CHECK_REJECT |
| 0784 | run_misc.at:5307: 784. access to OPTIONAL LINKAGE item not passed | checker | "cobrun: unsupported: CALL ""callee"": fewer USING args than the 2 parameters" | CANDIDATE_CHECK_REJECT |
| 0788 | run_misc.at:5406: 788. SYMBOLIC clause | grammar | cobrun: unsupported: not a numeric literal: Z-ASC | CANDIDATE_CHECK_REJECT |
| 0790 | run_misc.at:5480: 790. Computing of different USAGEs w/o decimal point | grammar | cobrun: unsupported: expected program name after PROGRAM-ID | CANDIDATE_CHECK_REJECT |
| 0791 | run_misc.at:6005: 791. Computing of different USAGEs w/- decimal point | grammar | cobrun: unsupported: expected program name after PROGRAM-ID | CANDIDATE_CHECK_REJECT |
| 0795 | run_misc.at:7063: 795. POINTER | checker | cobrun: unsupported: SET ADDRESS UP/DOWN BY: not a numeric index | CANDIDATE_CHECK_REJECT |
| 0797 | run_misc.at:7169: 797. ON EXCEPTION clause of DISPLAY | grammar | cobrun: unsupported: not a numeric literal: AT | CANDIDATE_CHECK_REJECT |
| 0798 | run_misc.at:7194: 798. EC-SCREEN-LINE-NUMBER and -STARTING-COLUMN | grammar | cobrun: unsupported: not a numeric literal: INVALID-LINE | CANDIDATE_CHECK_REJECT |
| 0800 | run_misc.at:7273: 800. SET LAST EXCEPTION TO OFF | grammar | cobrun: unsupported: not a numeric literal: FUNCTION | CANDIDATE_CHECK_REJECT |
| 0801 | run_misc.at:7309: 801. void PROCEDURE | grammar | cobrun: unsupported: verb RETURNING | CANDIDATE_CHECK_REJECT |
| 0802 | run_misc.at:7338: 802. Figurative constants to numeric field | checker |  | CANDIDATE_CHECK_REJECT |
| 0805 | "run_misc.at:7574: 805. void PROCEDURE, NOTHING return" | grammar | cobrun: unsupported: verb RETURNING | CANDIDATE_CHECK_REJECT |
| 0810 | run_misc.at:11291: 810. CALL with program prototypes | checker | "cobrun: unsupported: CALL ""D"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0811 | run_misc.at:11370: 811. REDEFINES values on FILLER and INITIALIZE | checker |  | CANDIDATE_CHECK_REJECT |
| 0812 | run_misc.at:11425: 812. PICTURE with constant-name | data-layout | cobc-rs: unsupported: PIC 9(FOO-BAR)9(FOO-BAR): BadRepeat | CANDIDATE_CHECK_REJECT |
| 0813 | run_misc.at:11453: 813. Quote marks in comment paragraphs | checker |  | CANDIDATE_CHECK_REJECT |
| 0814 | run_misc.at:11479: 814. Numeric MOVE with/without -fbinary-truncate | checker |  | CANDIDATE_CHECK_REJECT |
| 0815 | run_misc.at:11549: 815. Alphanumeric MOVE with truncation | name-resolution | "cobrun: undefined data name: X-LEFT," | CANDIDATE_PARSE_REJECT |
| 0816 | run_misc.at:11600: 816. PROGRAM-ID / CALL literal/variable with spaces | checker |  | CANDIDATE_CHECK_REJECT |
| 0817 | run_misc.at:11668: 817. PROGRAM-ID / CALL with hyphen and underscore | checker | "cobrun: unsupported: CALL ""_SUB-PROG_NOW"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0818 | run_misc.at:11705: 818. CALL with directory | grammar | cobrun: unsupported: expected program name after PROGRAM-ID | CANDIDATE_CHECK_REJECT |
| 0819 | run_misc.at:11763: 819. C-API (param based) | checker |  | CANDIDATE_CHECK_REJECT |
| 0820 | run_misc.at:11998: 820. C-API (field based) | checker |  | CANDIDATE_CHECK_REJECT |
| 0824 | run_misc.at:12434: 824. Default Arithmetic (1) | name-resolution | "cobrun: undefined data name: RSLTV2," | CANDIDATE_PARSE_REJECT |
| 0827 | run_misc.at:12706: 827. OSVS Arithmetic Test (2) | checker |  | CANDIDATE_CHECK_REJECT |
| 0828 | run_misc.at:12763: 828. SET CONSTANT directive | data-layout | cobrun: unsupported: unsupported level number $SET | CANDIDATE_CHECK_REJECT |
| 0831 | run_misc.at:12998: 831. 78 VALUE | grammar | cobrun: unsupported: not a numeric literal: DOGGY | CANDIDATE_CHECK_REJECT |
| 0833 | run_misc.at:13146: 833. DISPLAY UPON | checker |  | CANDIDATE_CHECK_REJECT |
| 0834 | run_misc.at:13248: 834. FLOAT-DECIMAL w/o SIZE ERROR | data-layout | cobrun: unsupported: unrecognized USAGE FLOAT-DECIMAL-16 | CANDIDATE_CHECK_REJECT |
| 0835 | run_misc.at:13422: 835. FLOAT-SHORT / FLOAT-LONG w/o SIZE ERROR | name-resolution | "cobrun: undefined data name: CMP1," | CANDIDATE_PARSE_REJECT |
| 0838 | run_misc.at:13730: 838. EC-SIZE-ZERO-DIVIDE | grammar | cobrun: unsupported: not a numeric literal: FUNCTION | CANDIDATE_CHECK_REJECT |
| 0839 | run_misc.at:13773: 839. EC-SIZE-OVERFLOW | grammar | cobrun: unsupported: not a numeric literal: FUNCTION | CANDIDATE_CHECK_REJECT |
| 0841 | run_misc.at:13893: 841. ENTRY FOR GO TO / GO TO ENTRY | checker |  | CANDIDATE_CHECK_REJECT |
| 0842 | run_misc.at:13983: 842. runtime checks within conditions | name-resolution | cobrun: undefined data name: | CANDIDATE_PARSE_REJECT |
| 0845 | run_misc.at:14292: 845. libcob version check | checker | cobrun: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 0846 | run_misc.at:14397: 846. assorted math | data-layout | cobrun: unsupported: unrecognized USAGE SIGNED-INT | CANDIDATE_CHECK_REJECT |
| 0848 | "run_file.at:23: 848. OPEN EXTEND and CLOSE, SEQUENTIAL" | semantic-check | cobrun: unsupported: OPEN: `FILE-OPT` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0850 | "run_file.at:154: 850. DELETE FILE, SEQUENTIAL" | semantic-check | cobrun: unsupported: OPEN: `FILE-OPT` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0852 | "run_file.at:261: 852. OPEN EXTEND and CLOSE, INDEXED" | semantic-check | cobrun: unsupported: OPEN: `FILE-OPT` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0853 | "run_file.at:333: 853. DELETE FILE, INDEXED" | semantic-check | cobrun: unsupported: OPEN: `FILE-OPT` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0865 | run_file.at:1036: 865. ASSIGN DYNAMIC with LOCAL-STORAGE item | data-layout | cobrun: unsupported: unsupported level number LOCAL-STORAGE | CANDIDATE_CHECK_REJECT |
| 0866 | "run_file.at:1073: 866. ASSIGN DYNAMIC with LOCAL-STORAGE item, INITIAL" | data-layout | cobrun: unsupported: unsupported level number LOCAL-STORAGE | CANDIDATE_CHECK_REJECT |
| 0867 | run_file.at:1113: 867. ASSIGN DYNAMIC with BASED data item | grammar | cobrun: unsupported: verb CHAINING | CANDIDATE_CHECK_REJECT |
| 0868 | run_file.at:1199: 868. ASSIGN DYNAMIC with data item in LINKAGE | name-resolution | cobrun: undefined data name: OMITTED | CANDIDATE_PARSE_REJECT |
| 0869 | run_file.at:1371: 869. ASSIGN DYNAMIC with empty data item | grammar | cobrun: unsupported: verb CHAINING | CANDIDATE_CHECK_REJECT |
| 0871 | run_file.at:1443: 871. INDEXED file key-name | grammar | cobrun: unsupported: expected program name after PROGRAM-ID | CANDIDATE_CHECK_REJECT |
| 0872 | run_file.at:1485: 872. INDEXED file sparse/split keys | data-layout | cobrun: unsupported: OCCURS count MAX-SUB is not an integer | CANDIDATE_CHECK_REJECT |
| 0873 | run_file.at:2005: 873. INDEXED file split keys WITH DUPLICATES | checker | cobrun: unsupported: INDEXED RECORD KEY `TEST-KEY-2` is not a field of the record | CANDIDATE_CHECK_REJECT |
| 0874 | run_file.at:2171: 874. INDEXED file variable length record | semantic-check | cobrun: unsupported: DELETE: `FILE` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0875 | run_file.at:2447: 875. INDEXED sample | data-layout | cobrun: unsupported: PIC 99X99: UnsupportedSymbol('X') | CANDIDATE_CHECK_REJECT |
| 0876 | run_file.at:2948: 876. WRITE + REWRITE FILE name | data-layout | cobrun: unsupported: OCCURS count MAX-SUB is not an integer | CANDIDATE_CHECK_REJECT |
| 0877 | run_file.at:3109: 877. START RELATIVE (1) | semantic-check | cobrun: unsupported: DELETE: `FILE` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0878 | run_file.at:3158: 878. START RELATIVE (2) | semantic-check | cobrun: unsupported: OPEN: `TEST-FILE` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0880 | run_file.at:3310: 880. READ on OPTIONAL missing RELATIVE / SEQUENTIAL | semantic-check | cobrun: unsupported: OPEN: `INFILE` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0881 | run_file.at:3432: 881. READ on OPTIONAL missing INDEXED file | semantic-check | cobrun: unsupported: DELETE: `FILE` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0882 | run_file.at:3482: 882. EXTERNAL RELATIVE file | checker | cobrun: unsupported: WRITE `TWO-RECORD`: not an FD record | CANDIDATE_CHECK_REJECT |
| 0883 | run_file.at:3546: 883. DECLARATIVES procedure referencing | grammar | cobrun: unsupported: verb END | CANDIDATE_CHECK_REJECT |
| 0885 | run_file.at:3630: 885. System routines for directories (1) | checker | "cobrun: unsupported: CALL ""CBL_CREATE_DIR"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0886 | run_file.at:3672: 886. System routines for directories (2) | checker | "cobrun: unsupported: CALL ""CBL_CREATE_DIR"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0887 | run_file.at:3769: 887. System routines for files | checker | "cobrun: unsupported: CALL ""CBL_CREATE_FILE"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0888 | run_file.at:3966: 888. System routines for files - filename mapping | grammar | cobrun: unsupported: verb CHAINING | CANDIDATE_CHECK_REJECT |
| 0889 | run_file.at:4083: 889. System routine CBL_COPY_FILE | checker | "cobrun: unsupported: CALL ""CBL_COPY_FILE"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0890 | run_file.at:4118: 890. Default file external name | checker | cobrun: unsupported: ACCEPT FROM DATE/TIME requires a pinned COB_CURRENT_DATE (the live clock is a non-claim) | CANDIDATE_CHECK_REJECT |
| 0891 | run_file.at:4192: 891. SEQUENTIAL basic I/O | semantic-check | cobrun: unsupported: DELETE: `FILE` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0892 | run_file.at:4241: 892. LINE SEQUENTIAL basic I/O | semantic-check | cobrun: unsupported: DELETE: `FILE` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0894 | run_file.at:4353: 894. LINE SEQUENTIAL record truncation (1) | semantic-check | cobrun: unsupported: condition: missing left operand | CANDIDATE_CHECK_REJECT |
| 0897 | run_file.at:4729: 897. LINAGE and LINAGE-COUNTER sample | grammar | cobrun: unsupported: not a numeric literal: ALL | CANDIDATE_CHECK_REJECT |
| 0901 | run_file.at:5455: 901. SEQUENTIAL file REWRITE | checker | cobrun: unsupported: SET EOF TO FALSE: the 88 has no `WHEN SET TO FALSE` value | CANDIDATE_CHECK_REJECT |
| 0905 | run_file.at:5775: 905. SEQUENTIAL file with SHARING READ ONLY | checker | "cobrun: unsupported: CALL ""SYSTEM"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0907 | run_file.at:5923: 907. RELATIVE SEQUENTIAL basic I/O | semantic-check | cobrun: unsupported: DELETE: `FILE` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0908 | run_file.at:5957: 908. RELATIVE RANDOM basic I/O | semantic-check | cobrun: unsupported: DELETE: `FILE` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0909 | run_file.at:6007: 909. RELATIVE SEQUENTIAL with variable records | checker | cobrun: unsupported: RELATIVE file `F` has no RELATIVE KEY | CANDIDATE_CHECK_REJECT |
| 0910 | run_file.at:6081: 910. INDEXED SEQUENTIAL basic I/O | checker | cobrun: unsupported: trailing tokens in condition at ) | CANDIDATE_CHECK_REJECT |
| 0911 | run_file.at:6119: 911. INDEXED SEQUENTIAL with variable records | checker | cobrun: unsupported: reference-modification length 'REC-SIZE - 2' is not an integer | CANDIDATE_CHECK_REJECT |
| 0918 | run_file.at:6696: 918. INDEXED file with LOCK AUTOMATIC (2) | checker | "cobrun: unsupported: CALL ""SYSTEM"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 0921 | run_file.at:6957: 921. INDEXED partial keys | semantic-check | cobrun: unsupported: DELETE: `FILE` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0936 | run_file.at:11235: 936. RELATIVE Multi-Record | data-layout | cobrun: unsupported: OCCURS count MAX-SUB is not an integer | CANDIDATE_CHECK_REJECT |
| 0937 | run_file.at:11485: 937. INDEXED File READ/DELETE/READ | data-layout | cobrun: unsupported: OCCURS count MAX-SUB is not an integer | CANDIDATE_CHECK_REJECT |
| 0939 | run_file.at:12388: 939. LINE SEQUENTIAL REWRITE | data-layout | cobrun: unsupported: OCCURS count MAX-SUB is not an integer | CANDIDATE_CHECK_REJECT |
| 0940 | run_file.at:12660: 940. LINE SEQUENTIAL data | data-layout | cobrun: unsupported: OCCURS count MAX-SUB is not an integer | CANDIDATE_CHECK_REJECT |
| 0941 | run_file.at:12817: 941. Concatenated Files | name-resolution | cobrun: undefined data name: ENVIRONMENT | CANDIDATE_PARSE_REJECT |
| 0952 | run_file.at:13617: 952. Scope of FD GLOBAL in nested programs | semantic-check | cobrun: unsupported: OPEN: `FILE-EXT` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0953 | run_file.at:13774: 953. OPEN / CLOSE with multiple filenames | data-layout | cobrun: unsupported: unsupported level number TAT-FILE1 | CANDIDATE_CHECK_REJECT |
| 0954 | run_reportwriter.at:23: 954. Report Line Order | data-layout | cobrun: unsupported: PIC : Empty | CANDIDATE_CHECK_REJECT |
| 0957 | run_reportwriter.at:258: 957. EMPTY REPORT | data-layout | cobrun: unsupported: PIC : Empty | CANDIDATE_CHECK_REJECT |
| 0958 | run_reportwriter.at:327: 958. PAGE LIMIT REPORT | grammar | "cobrun: unsupported: not a numeric literal: FOO," | CANDIDATE_CHECK_REJECT |
| 0960 | run_reportwriter.at:460: 960. Sample Customer Report | semantic-check | "cobrun: unsupported: OPEN: `TRANSACTION-DATA,` is not a declared file" | CANDIDATE_CHECK_REJECT |
| 0961 | run_reportwriter.at:775: 961. Sample Charge Report | semantic-check | "cobrun: unsupported: OPEN: `TRANSACTION-DATA,` is not a declared file" | CANDIDATE_CHECK_REJECT |
| 0962 | run_reportwriter.at:1128: 962. Sample Charge Report 2 | semantic-check | "cobrun: unsupported: OPEN: `TRANSACTION-DATA,` is not a declared file" | CANDIDATE_CHECK_REJECT |
| 0963 | run_reportwriter.at:1498: 963. Sample Charge Report 3 | semantic-check | "cobrun: unsupported: OPEN: `TRANSACTION-DATA,` is not a declared file" | CANDIDATE_CHECK_REJECT |
| 0964 | run_reportwriter.at:1798: 964. Sample Charge Report 4 | semantic-check | "cobrun: unsupported: OPEN: `SALES-DATA,` is not a declared file" | CANDIDATE_CHECK_REJECT |
| 0965 | run_reportwriter.at:2214: 965. Sample Payroll Report | semantic-check | "cobrun: unsupported: OPEN: `PAYROLL-REGISTER-DATA,` is not a declared file" | CANDIDATE_CHECK_REJECT |
| 0966 | run_reportwriter.at:2895: 966. Sample REPORT with RIGHT/CENTER | checker |  | CANDIDATE_CHECK_REJECT |
| 0967 | run_reportwriter.at:3063: 967. STUDENT REPORT with INITIAL | semantic-check | cobrun: unsupported: OPEN: `OUTPUT` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0968 | run_reportwriter.at:3215: 968. ORDER REPORT; Test substring | semantic-check | cobrun: unsupported: OPEN: `OUTPUT` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0969 | run_reportwriter.at:3563: 969. Sample Control Break | semantic-check | cobrun: unsupported: OPEN: `OUTPUT` is not a declared file | CANDIDATE_CHECK_REJECT |
| 0970 | run_reportwriter.at:3792: 970. Sample Inventory Report | checker |  | CANDIDATE_CHECK_REJECT |
| 0972 | run_reportwriter.at:4113: 972. Report with OCCURS | name-resolution | cobrun: undefined data name: NUMS | CANDIDATE_PARSE_REJECT |
| 0975 | run_reportwriter.at:4580: 975. Duplicate INITIATE | grammar | "cobrun: unsupported: not a numeric literal: FOO," | CANDIDATE_CHECK_REJECT |
| 0976 | run_reportwriter.at:4646: 976. Missing INITIATE and GENERATE | grammar | "cobrun: unsupported: not a numeric literal: FOO," | CANDIDATE_CHECK_REJECT |
| 0978 | run_reportwriter.at:4760: 978. Next Group Next Page | semantic-check | cobrun: unsupported: condition: unrecognized relational operator (expected = > < >= <= <> GREATER LESS EQUAL) | CANDIDATE_CHECK_REJECT |
| 0981 | run_reportwriter.at:9519: 981. BEFORE REPORTING | name-resolution | "cobrun: undefined data name: WS-TRANSPORT-PAY," | CANDIDATE_PARSE_REJECT |
| 0990 | run_functions.at:165: 990. FUNCTION BYTE-LENGTH | checker |  | CANDIDATE_CHECK_REJECT |
| 0991 | run_functions.at:221: 991. FUNCTION CHAR | name-resolution | cobrun: undefined data name: FUNCTION | CANDIDATE_PARSE_REJECT |
| 0993 | run_functions.at:284: 993. FUNCTION CONCAT / CONCATENATE | name-resolution | cobrun: undefined data name: FUNCTION | CANDIDATE_PARSE_REJECT |
| 0995 | run_functions.at:359: 995. FUNCTION BIT-OF and BIT-TO-CHAR | checker | cobrun: unsupported: MOVE ALL: expected a non-empty literal or figurative | CANDIDATE_CHECK_REJECT |
| 0996 | run_functions.at:416: 996. FUNCTION HEX-OF and HEX-TO-CHAR | data-layout | "cobrun: unsupported: USAGE NATIONAL: cobc 3.2 leaves NATIONAL unfinished (-Wunfinished) -- a non-claim, not a buildable front-end gap" | CANDIDATE_CHECK_REJECT |
| 0997 | run_functions.at:547: 997. FUNCTION CONTENT-LENGTH | grammar | cobrun: unsupported: not a numeric literal: Z | CANDIDATE_CHECK_REJECT |
| 0998 | run_functions.at:581: 998. FUNCTION CONTENT-OF | grammar | cobrun: unsupported: not a numeric literal: Z | CANDIDATE_CHECK_REJECT |
| 0999 | run_functions.at:656: 999. FUNCTION as CALL parameter BY CONTENT | name-resolution | cobrun: undefined data name: FUNCTION | CANDIDATE_PARSE_REJECT |
| 1002 | run_functions.at:739: 1002. FUNCTION CURRENT-DATE | name-resolution | cobrun: undefined data name: FUNCTION | CANDIDATE_PARSE_REJECT |
| 1004 | run_functions.at:835: 1004. FUNCTION DATE-TO-YYYYMMDD | checker | cobrun: unsupported: FUNCTION DATE-TO-YYYYMMDD requires a pinned COB_CURRENT_DATE | CANDIDATE_CHECK_REJECT |
| 1006 | run_functions.at:885: 1006. FUNCTION DAY-TO-YYYYDDD | checker | cobrun: unsupported: FUNCTION DAY-TO-YYYYDDD requires a pinned COB_CURRENT_DATE | CANDIDATE_CHECK_REJECT |
| 1025 | run_functions.at:1642: 1025. FUNCTION INTEGER | grammar | cobrun: unsupported: not a numeric literal: / | CANDIDATE_CHECK_REJECT |
| 1030 | run_functions.at:1801: 1030. FUNCTION LENGTH | data-layout | cobrun: unsupported: PIC N(9): UnsupportedSymbol('N') | CANDIDATE_CHECK_REJECT |
| 1034 | run_functions.at:1965: 1034. FUNCTION LOCALE-TIME-FROM-SECONDS | checker | cobrun: unsupported: FUNCTION LOCALE-TIME-FROM-SECONDS: cobc 3.2 does not implement it (a compile-reject) or it is a live-clock/locale/GMP-PRNG non-claim | CANDIDATE_CHECK_REJECT |
| 1037 | run_functions.at:2039: 1037. FUNCTION LOWER-CASE | grammar | cobrun: unsupported: not a numeric literal: ALL | CANDIDATE_CHECK_REJECT |
| 1046 | run_functions.at:2286: 1046. FUNCTION MOD (invalid) | grammar | cobrun: unsupported: not a numeric literal: FUNCTION | CANDIDATE_CHECK_REJECT |
| 1051 | run_functions.at:2422: 1051. FUNCTION MODULE-PATH | checker | cobrun: unsupported: FUNCTION MODULE-PATH: cobc 3.2 does not implement it (a compile-reject) or it is a live-clock/locale/GMP-PRNG non-claim | CANDIDATE_CHECK_REJECT |
| 1054 | run_functions.at:2493: 1054. FUNCTION MONETARY-DECIMAL-POINT | checker | cobrun: unsupported: FUNCTION MONETARY-DECIMAL-POINT: cobc 3.2 does not implement it (a compile-reject) or it is a live-clock/locale/GMP-PRNG non-claim | CANDIDATE_CHECK_REJECT |
| 1055 | run_functions.at:2516: 1055. FUNCTION MONETARY-THOUSANDS-SEPARATOR | checker | cobrun: unsupported: FUNCTION MONETARY-THOUSANDS-SEPARATOR: cobc 3.2 does not implement it (a compile-reject) or it is a live-clock/locale/GMP-PRNG non-claim | CANDIDATE_CHECK_REJECT |
| 1056 | run_functions.at:2539: 1056. FUNCTION NUMERIC-DECIMAL-POINT | checker | cobrun: unsupported: FUNCTION NUMERIC-DECIMAL-POINT: cobc 3.2 does not implement it (a compile-reject) or it is a live-clock/locale/GMP-PRNG non-claim | CANDIDATE_CHECK_REJECT |
| 1057 | run_functions.at:2562: 1057. FUNCTION NUMERIC-THOUSANDS-SEPARATOR | checker | cobrun: unsupported: FUNCTION NUMERIC-THOUSANDS-SEPARATOR: cobc 3.2 does not implement it (a compile-reject) or it is a live-clock/locale/GMP-PRNG non-claim | CANDIDATE_CHECK_REJECT |
| 1067 | run_functions.at:2888: 1067. FUNCTION RANDOM | checker | cobrun: unsupported: FUNCTION RANDOM: cobc 3.2 does not implement it (a compile-reject) or it is a live-clock/locale/GMP-PRNG non-claim | CANDIDATE_CHECK_REJECT |
| 1070 | run_functions.at:2954: 1070. FUNCTION REM (invalid) | grammar | cobrun: unsupported: not a numeric literal: FUNCTION | CANDIDATE_CHECK_REJECT |
| 1080 | run_functions.at:3238: 1080. FUNCTION SUBSTITUTE | grammar | cobrun: unsupported: not a numeric literal: ALL | CANDIDATE_CHECK_REJECT |
| 1089 | run_functions.at:3531: 1089. FUNCTION TEST-FORMATTED-DATETIME with times | grammar | cobrun: unsupported: not a numeric literal: SPACE | CANDIDATE_CHECK_REJECT |
| 1090 | run_functions.at:3612: 1090. FUNCTION TEST-FORMATTED-DATETIME with datetimes | grammar | cobrun: unsupported: not a numeric literal: SPACE | CANDIDATE_CHECK_REJECT |
| 1096 | run_functions.at:4083: 1096. FUNCTION TRIM with reference modding | grammar | cobrun: unsupported: not a numeric literal: (2 | CANDIDATE_CHECK_REJECT |
| 1101 | run_functions.at:4216: 1101. FUNCTION WHEN-COMPILED | grammar | cobrun: unsupported: not a numeric literal: 17:5 | CANDIDATE_CHECK_REJECT |
| 1102 | run_functions.at:4270: 1102. FUNCTION YEAR-TO-YYYY | checker | cobrun: unsupported: FUNCTION YEAR-TO-YYYY requires a pinned COB_CURRENT_DATE | CANDIDATE_CHECK_REJECT |
| 1103 | run_functions.at:4294: 1103. Formatted funcs w/ invalid variable format | checker |  | CANDIDATE_CHECK_REJECT |
| 1104 | run_functions.at:4375: 1104. FORMATTED-(DATE)TIME with SYSTEM-OFFSET | grammar | cobrun: unsupported: not a numeric literal: SYSTEM-OFFSET | CANDIDATE_CHECK_REJECT |
| 1107 | run_functions.at:4457: 1107. User-Defined FUNCTION with/without parameter | grammar | cobrun: unsupported: not a numeric literal: WITHPAR(1) | CANDIDATE_CHECK_REJECT |
| 1108 | run_functions.at:4508: 1108. UDF in COMPUTE | checker | cobrun: unsupported: FUNCTION FUNC: cobc 3.2 does not implement it (a compile-reject) or it is a live-clock/locale/GMP-PRNG non-claim | CANDIDATE_CHECK_REJECT |
| 1111 | run_extensions.at:25: 1111. CALL BY CONTENT binary and literal | checker | "cobrun: unsupported: CALL ""dump"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1112 | run_extensions.at:72: 1112. Numeric Boolean literals | grammar | cobrun: unsupported: not a numeric literal: B | CANDIDATE_CHECK_REJECT |
| 1115 | run_extensions.at:188: 1115. Hexadecimal numeric literals | grammar | cobrun: unsupported: not a numeric literal: H | CANDIDATE_CHECK_REJECT |
| 1117 | run_extensions.at:237: 1117. ADDRESS OF | grammar | cobrun: unsupported: not a numeric literal: NULL | CANDIDATE_CHECK_REJECT |
| 1118 | run_extensions.at:287: 1118. LENGTH OF | grammar | cobrun: unsupported: not a numeric literal: LENGTH | CANDIDATE_CHECK_REJECT |
| 1119 | run_extensions.at:451: 1119. SET TO SIZE OF | grammar | cobrun: unsupported: not a numeric literal: SIZE | CANDIDATE_CHECK_REJECT |
| 1120 | run_extensions.at:488: 1120. WHEN-COMPILED | grammar | cobrun: unsupported: not a numeric literal: WHEN-COMPILED | CANDIDATE_CHECK_REJECT |
| 1127 | run_extensions.at:846: 1127. OCCURS UNBOUNDED (1) | data-layout | cobrun: unsupported: OCCURS max UNBOUNDED is not an integer | CANDIDATE_CHECK_REJECT |
| 1128 | run_extensions.at:908: 1128. OCCURS UNBOUNDED (2) | data-layout | cobrun: unsupported: OCCURS max UNBOUNDED is not an integer | CANDIDATE_CHECK_REJECT |
| 1129 | run_extensions.at:1048: 1129. INITIALIZE OCCURS UNBOUNDED | data-layout | cobrun: unsupported: OCCURS max UNBOUNDED is not an integer | CANDIDATE_CHECK_REJECT |
| 1132 | run_extensions.at:1463: 1132. DEPENDING ON with ODOSLIDE for IBM | name-resolution | "cobrun: undefined data name: L1-3-2(1," | CANDIDATE_PARSE_REJECT |
| 1133 | run_extensions.at:1568: 1133. INITIALIZE level 01 OCCURS | name-resolution | cobrun: undefined data name: DEFAULT | CANDIDATE_PARSE_REJECT |
| 1134 | run_extensions.at:1625: 1134. MOVE of non-integer to alphanumeric | checker |  | CANDIDATE_CHECK_REJECT |
| 1135 | run_extensions.at:1715: 1135. CALL USING file-name | checker | "cobrun: unsupported: CALL ""setfilename"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1136 | run_extensions.at:1761: 1136. CALL unusual PROGRAM-ID. | grammar | cobrun: unsupported: expected program name after PROGRAM-ID | CANDIDATE_CHECK_REJECT |
| 1137 | run_extensions.at:1826: 1137. CALL / GOBACK with LOCAL-STORAGE | checker | cobrun: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 1138 | run_extensions.at:1877: 1138. CALL BY VALUE alphanumeric item | checker |  | CANDIDATE_CHECK_REJECT |
| 1139 | run_extensions.at:1914: 1139. CALL BY VALUE numeric literal with SIZE IS | name-resolution | cobrun: undefined data name: 4 | CANDIDATE_PARSE_REJECT |
| 1140 | run_extensions.at:2019: 1140. CALL BY VALUE to C | checker | cobrun: unsupported: expected data name after a level number | CANDIDATE_CHECK_REJECT |
| 1142 | run_extensions.at:2126: 1142. Quoted PROGRAM-ID | grammar | cobrun: unsupported: expected program name after PROGRAM-ID | CANDIDATE_CHECK_REJECT |
| 1143 | run_extensions.at:2149: 1143. PROGRAM-ID AS clause | checker | "cobrun: unsupported: CALL ""prog"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1146 | run_extensions.at:2285: 1146. Redefining TALLY | checker |  | CANDIDATE_CHECK_REJECT |
| 1147 | run_extensions.at:2321: 1147. PROCEDURE DIVISION USING BY ... | checker |  | CANDIDATE_CHECK_REJECT |
| 1148 | run_extensions.at:2374: 1148. PROCEDURE DIVISION CHAINING | grammar | cobrun: unsupported: verb CHAINING | CANDIDATE_CHECK_REJECT |
| 1161 | run_extensions.at:3097: 1161. Obsolete 2002 keywords with COBOL2014 | checker |  | CANDIDATE_CHECK_REJECT |
| 1162 | run_extensions.at:3127: 1162. System routine with wrong number of parameters | checker |  | CANDIDATE_CHECK_REJECT |
| 1163 | run_extensions.at:3170: 1163. System routine C$NARG | checker | "cobrun: unsupported: CALL ""C$NARG"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1164 | run_extensions.at:3247: 1164. System routine C$PARAMSIZE | checker | "cobrun: unsupported: CALL ""C$PARAMSIZE"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1165 | run_extensions.at:3289: 1165. System routine C$CALLEDBY | checker | "cobrun: unsupported: CALL ""C$CALLEDBY"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1166 | run_extensions.at:3335: 1166. System routine C$JUSTIFY | checker | "cobrun: unsupported: CALL ""C$JUSTIFY"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1167 | run_extensions.at:3360: 1167. System routine C$PRINTABLE | checker | "cobrun: unsupported: CALL ""C$PRINTABLE"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1168 | run_extensions.at:3389: 1168. System routine C$MAKEDIR | checker | "cobrun: unsupported: CALL ""C$MAKEDIR"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1169 | run_extensions.at:3410: 1169. System routine C$GETPID | checker | "cobrun: unsupported: CALL ""C$GETPID"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1170 | run_extensions.at:3435: 1170. System routine C$TOUPPER | checker | "cobrun: unsupported: CALL ""C$TOUPPER"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1171 | run_extensions.at:3460: 1171. System routine C$TOLOWER | checker | "cobrun: unsupported: CALL ""C$TOLOWER"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1172 | run_extensions.at:3485: 1172. System routine CBL_OR | grammar | cobrun: unsupported: not a numeric literal: X | CANDIDATE_CHECK_REJECT |
| 1173 | run_extensions.at:3512: 1173. System routine CBL_NOR | grammar | cobrun: unsupported: not a numeric literal: X | CANDIDATE_CHECK_REJECT |
| 1174 | run_extensions.at:3539: 1174. System routine CBL_AND | checker | "cobrun: unsupported: CALL ""CBL_AND"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1175 | run_extensions.at:3566: 1175. System routine CBL_XOR | grammar | cobrun: unsupported: not a numeric literal: X | CANDIDATE_CHECK_REJECT |
| 1176 | run_extensions.at:3593: 1176. System routine CBL_IMP | checker | "cobrun: unsupported: CALL ""CBL_IMP"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1177 | run_extensions.at:3620: 1177. System routine CBL_NIMP | checker | "cobrun: unsupported: CALL ""CBL_NIMP"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1178 | run_extensions.at:3647: 1178. System routine CBL_NOT | checker | "cobrun: unsupported: CALL ""CBL_NOT"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1179 | run_extensions.at:3673: 1179. System routine CBL_EQ | checker | "cobrun: unsupported: CALL ""CBL_EQ"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1180 | run_extensions.at:3700: 1180. System routine CBL_GC_GETOPT | grammar | cobrun: unsupported: not a numeric literal: NULL | CANDIDATE_CHECK_REJECT |
| 1181 | run_extensions.at:4137: 1181. System routine CBL_GC_FORK | checker | "cobrun: unsupported: CALL ""C$GETPID"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1183 | run_extensions.at:4257: 1183. System routine CBL_GC_HOSTED | data-layout | cobrun: unsupported: unrecognized USAGE BINARY-C-LONG | CANDIDATE_CHECK_REJECT |
| 1184 | "run_extensions.at:4387: 1184. System routine SYSTEM, parameter handling" | grammar | cobrun: unsupported: verb CHAINING | CANDIDATE_CHECK_REJECT |
| 1185 | run_extensions.at:4498: 1185. System routine CBL_EXIT_PROC | data-layout | cobrun: unsupported: unrecognized USAGE PROCEDURE-POINTER | CANDIDATE_CHECK_REJECT |
| 1186 | run_extensions.at:4608: 1186. System routine CBL_ERROR_PROC (1) | checker | cobrun: unsupported: SET ERR-PROC-ADDRESS TO <value>: target is not a numeric/index item | CANDIDATE_CHECK_REJECT |
| 1187 | run_extensions.at:4707: 1187. System routine CBL_ERROR_PROC (2) | data-layout | cobrun: unsupported: unsupported level number LOCAL-STORAGE | CANDIDATE_CHECK_REJECT |
| 1189 | run_extensions.at:4804: 1189. CALL own PROGRAM-ID and RECURSIVE attribute | checker |  | CANDIDATE_CHECK_REJECT |
| 1190 | run_extensions.at:4869: 1190. DISPLAY DIRECTIVE and $DISPLAY | checker |  | CANDIDATE_CHECK_REJECT |
| 1195 | run_extensions.at:5006: 1195. Invalid source format | checker |  | CANDIDATE_CHECK_REJECT |
| 1197 | run_extensions.at:5074: 1197. COBOLX format | checker | cobrun: unsupported: no PROCEDURE DIVISION | CANDIDATE_CHECK_REJECT |
| 1203 | run_extensions.at:5363: 1203. EXHIBIT statement | name-resolution | cobrun: undefined data name: SORT-RETURN | CANDIDATE_PARSE_REJECT |
| 1206 | run_extensions.at:5571: 1206. GCOS floating-point usages | data-layout | cobrun: unsupported: unrecognized USAGE COMPUTATIONAL-9 | CANDIDATE_CHECK_REJECT |
| 1207 | run_extensions.at:5610: 1207. PICTURE L (basic) | data-layout | cobrun: unsupported: PIC LX(10): UnsupportedSymbol('L') | CANDIDATE_CHECK_REJECT |
| 1208 | run_extensions.at:5740: 1208. PICTURE L (under/over shoot) | data-layout | cobrun: unsupported: PIC LX(9): UnsupportedSymbol('L') | CANDIDATE_CHECK_REJECT |
| 1209 | run_extensions.at:5795: 1209. PICTURE L (MOVE CORRESPONDING) | data-layout | cobrun: unsupported: PIC LX(5): UnsupportedSymbol('L') | CANDIDATE_CHECK_REJECT |
| 1210 | run_extensions.at:5873: 1210. PICTURE L (OCCURS ... PIC L) | data-layout | cobrun: unsupported: PIC LX(3): UnsupportedSymbol('L') | CANDIDATE_CHECK_REJECT |
| 1211 | run_extensions.at:5942: 1211. PICTURE L (REDEFINES) | data-layout | cobrun: unsupported: PIC LX(5): UnsupportedSymbol('L') | CANDIDATE_CHECK_REJECT |
| 1212 | run_extensions.at:6030: 1212. INSPECT TRAILING | checker | cobrun: unsupported: INSPECT TALLYING FOR: unrecognized mode `TRAILING` (expected ALL/LEADING/CHARACTERS) | CANDIDATE_CHECK_REJECT |
| 1213 | run_extensions.at:6100: 1213. INSPECT REPLACING TRAILING ZEROS BY SPACES | checker | cobrun: unsupported: INSPECT REPLACING: unrecognized mode `TRAILING` (expected CHARACTERS/ALL/LEADING/FIRST) | CANDIDATE_CHECK_REJECT |
| 1214 | run_extensions.at:6122: 1214. INSPECT REPLACING complex | checker | cobrun: unsupported: INSPECT region clause near Str([66]) | CANDIDATE_CHECK_REJECT |
| 1220 | run_ml.at:204: 1220. XML GENERATE exceptions | grammar | cobrun: unsupported: not a numeric literal: X | CANDIDATE_CHECK_REJECT |
| 1222 | run_ml.at:360: 1222. XML GENERATE trimming | data-layout | cobrun: unsupported: PIC VPP99: ScalingPDeferred | CANDIDATE_CHECK_REJECT |
| 1225 | run_ml.at:532: 1225. JSON GENERATE general | grammar | cobrun: unsupported: not a numeric literal: ALL | CANDIDATE_CHECK_REJECT |
| 1227 | run_ml.at:628: 1227. JSON GENERATE exceptions | grammar | cobrun: unsupported: not a numeric literal: JSON-CODE | CANDIDATE_CHECK_REJECT |
| 1229 | run_ml.at:737: 1229. JSON GENERATE trimming | data-layout | cobrun: unsupported: PIC VPP99: ScalingPDeferred | CANDIDATE_CHECK_REJECT |
| 1244 | data_binary.at:1409: 1244. MOVE DISPLAY to BINARY | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 1245 | data_binary.at:1595: 1245. MOVE PACKED-DECIMAL to BINARY | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 1246 | data_binary.at:1781: 1246. MOVE BINARY to PACKED-DECIMAL | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 1247 | data_binary.at:1961: 1247. MOVE BINARY to BINARY | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 1248 | data_binary.at:2143: 1248. PPP COMP-5 | data-layout | cobrun: unsupported: PIC VPPP999: ScalingPDeferred | CANDIDATE_CHECK_REJECT |
| 1253 | data_display.at:172: 1253. DISPLAY: unsigned | checker | cobrun: unsupported: a group MOVE is distributed across its leaves by write_field | CANDIDATE_CHECK_REJECT |
| 1254 | data_display.at:226: 1254. MOVE DISPLAY to DISPLAY | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 1255 | data_display.at:408: 1255. PPP DISPLAY | data-layout | cobrun: unsupported: PIC VPPP999: ScalingPDeferred | CANDIDATE_CHECK_REJECT |
| 1257 | data_display.at:533: 1257. DISPLAY: ADD and SUBTRACT w/o SIZE ERROR | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 1258 | "data_display.at:16940: 1258. DISPLAY: ADD and SUBTRACT, all ROUNDED MODEs" | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 1259 | data_packed.at:27: 1259. PACKED-DECIMAL dump | checker | "cobrun: unsupported: CALL ""dump"": not a contained program (external CALL is a boundary)" | CANDIDATE_CHECK_REJECT |
| 1261 | data_packed.at:221: 1261. PACKED-DECIMAL used with MOVE | grammar | cobrun: unsupported: not a numeric literal: FENCE | CANDIDATE_CHECK_REJECT |
| 1262 | data_packed.at:459: 1262. MOVE PACKED-DECIMAL to PACKED-DECIMAL | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 1263 | data_packed.at:639: 1263. MOVE PACKED-DECIMAL to DISPLAY | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 1264 | data_packed.at:825: 1264. MOVE DISPLAY to PACKED-DECIMAL | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 1266 | data_packed.at:1063: 1266. PACKED-DECIMAL arithmetic | grammar | cobrun: unsupported: not a numeric literal: FENCE | CANDIDATE_CHECK_REJECT |
| 1267 | data_packed.at:1187: 1267. PACKED-DECIMAL comparison | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 1268 | data_packed.at:1269: 1268. PACKED-DECIMAL numeric test (1) | grammar | cobrun: unsupported: not a numeric literal: X | CANDIDATE_CHECK_REJECT |
| 1269 | data_packed.at:1333: 1269. PACKED-DECIMAL numeric test (2) | grammar | cobrun: unsupported: not a numeric literal: X | CANDIDATE_CHECK_REJECT |
| 1272 | data_packed.at:1474: 1272. COMP-6 arithmetic | grammar | cobrun: unsupported: not a numeric literal: FENCE | CANDIDATE_CHECK_REJECT |
| 1273 | data_packed.at:1563: 1273. COMP-6 numeric | grammar | cobrun: unsupported: not a numeric literal: X | CANDIDATE_CHECK_REJECT |
| 1274 | data_packed.at:1609: 1274. COMP-6 comparison | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 1275 | data_packed.at:1671: 1275. COMP-3 vs. COMP-6 - BCD comparison | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 1276 | data_packed.at:1971: 1276. PPP COMP-3 | grammar | cobrun: unsupported: not a numeric literal: FENCE | CANDIDATE_CHECK_REJECT |
| 1277 | data_packed.at:2083: 1277. PPP COMP-6 | grammar | cobrun: unsupported: not a numeric literal: FENCE | CANDIDATE_CHECK_REJECT |
| 1278 | data_packed.at:2174: 1278. arithmetic truncation with USAGE PACKED-DECIMAL | grammar | cobrun: unsupported: not a numeric literal: FENCE | CANDIDATE_CHECK_REJECT |
| 1279 | data_packed.at:2236: 1279. MOVE between several BCD fields | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 1280 | data_packed.at:12326: 1280. BCD ADD and SUBTRACT w/o SIZE ERROR | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 1281 | "data_packed.at:28746: 1281. BCD ADD and SUBTRACT, all ROUNDED MODEs" | data-layout | cobrun: unsupported: unsupported level number REPLACE | CANDIDATE_CHECK_REJECT |
| 1282 | data_pointer.at:21: 1282. POINTER: display | grammar | cobrun: unsupported: not a numeric literal: NULL | CANDIDATE_CHECK_REJECT |

683 rows; generated by `gnucobol-rs-testsuite reject-census generate` — do not edit by hand.
