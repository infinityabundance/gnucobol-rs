/* Oracle harness for cobgetopt.c (cob_getopt_long_long). Reads scenarios from stdin, one per line:
 *
 *   LABEL <TAB> long_only <TAB> optstring <TAB> longspec <TAB> arg1 arg2 ...
 *
 * longspec is '-' (no long options) or 'name:has_arg:val|name:has_arg:val|...' (flag always NULL —
 * GnuCOBOL's own long_options tables never use a non-NULL flag). args are space-separated and become
 * argv[1..]; argv[0] is a fixed "prog". For each scenario the harness drives cob_getopt_long_long to
 * completion and prints:  LABEL  r:optarg:optind:optopt  r:...  ...  (one token per call, optarg '-' = NULL,
 * final token has r=-1). cob_opterr is forced to 0 so only the parse result is compared, not stderr text.
 *
 * Build: gcc -O2 -I$PREFIX/include getopt_harness.c -o getopt_harness -L$PREFIX/lib -lcob
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <libcob.h>
#include <libcob/cobgetopt.h>

#define MAXARG 64
#define MAXOPT 64

int
main (void)
{
	char line[4096];

	while (fgets (line, sizeof line, stdin)) {
		size_t n = strlen (line);
		while (n && (line[n - 1] == '\n' || line[n - 1] == '\r')) {
			line[--n] = '\0';
		}
		if (n == 0 || line[0] == '#') {
			continue;
		}

		/* split on tabs into: label, long_only, optstring, longspec, args */
		char *fields[5];
		int nf = 0;
		char *p = line;
		for (nf = 0; nf < 5; nf++) {
			fields[nf] = p;
			char *t = strchr (p, '\t');
			if (!t) { nf++; break; }
			*t = '\0';
			p = t + 1;
		}
		if (nf < 5) {
			/* allow an empty args field */
			if (nf == 4) { fields[4] = (char *)""; nf = 5; }
			else continue;
		}

		const char *label = fields[0];
		int long_only = atoi (fields[1]);
		const char *optstring = fields[2];
		char *longspec = fields[3];
		char *argspec = fields[4];

		/* build argv */
		char *argv[MAXARG];
		int argc = 0;
		argv[argc++] = (char *)"prog";
		{
			char *save = NULL;
			char *tok = strtok_r (argspec, " ", &save);
			while (tok && argc < MAXARG - 1) {
				argv[argc++] = tok;
				tok = strtok_r (NULL, " ", &save);
			}
		}
		argv[argc] = NULL;

		/* build longopts */
		struct option longopts[MAXOPT];
		int no = 0;
		if (strcmp (longspec, "-") != 0) {
			char *save = NULL;
			char *tok = strtok_r (longspec, "|", &save);
			while (tok && no < MAXOPT - 1) {
				/* name:has_arg:val */
				char *c1 = strchr (tok, ':');
				char *c2 = c1 ? strchr (c1 + 1, ':') : NULL;
				if (c1 && c2) {
					*c1 = '\0'; *c2 = '\0';
					longopts[no].name = tok;
					longopts[no].has_arg = atoi (c1 + 1);
					longopts[no].flag = NULL;
					longopts[no].val = atoi (c2 + 1);
					no++;
				}
				tok = strtok_r (NULL, "|", &save);
			}
		}
		longopts[no].name = NULL;
		longopts[no].has_arg = 0;
		longopts[no].flag = NULL;
		longopts[no].val = 0;

		/* drive the scanner */
		cob_optind = 0;	/* force _getopt_initialize of the file-static state */
		cob_opterr = 0;
		cob_optopt = '?';
		cob_optarg = NULL;

		printf ("%s", label);
		int guard = 0;
		for (;;) {
			int r = cob_getopt_long_long (argc, argv, optstring,
				no ? longopts : NULL, NULL, long_only);
			const char *oa = cob_optarg ? cob_optarg : "-";
			printf (" %d:%s:%d:%d", r, oa, cob_optind, cob_optopt);
			if (r == -1 || ++guard > 50) {
				break;
			}
		}
		printf ("\n");
	}
	return 0;
}
